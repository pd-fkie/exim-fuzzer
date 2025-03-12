use libafl::prelude::{
    HitcountsMapObserver,
    StdMapObserver,
    CanTrack,
    TimeObserver,
    MaxMapFeedback,
    CalibrationStage,
    feedback_or,
    TimeFeedback,
    CrashFeedback,
    TimeoutFeedback,
    StdState,
    InMemoryOnDiskCorpus,
    OnDiskCorpus,
    Error,
    StdFuzzer,
    Fuzzer,
    Tokens,
    HasMetadata,
    StdScheduledMutator,
    QueueScheduler,
    StdMutationalStage,
    LlmpRestartingEventManager,
    Launcher,
    EventConfig,
    OnDiskJsonMonitor,
    Input,
    ClientDescription,
    feedback_and_fast,
    Evaluator,
    Corpus,
    HasCorpus,
};
use libafl_bolts::prelude::{
    ShMem,
    ShMemProvider,
    UnixShMemProvider,
    AsSliceMut,
    StdRand,
    tuple_list,
    current_nanos,
    StdShMemProvider,
    Cores,
    current_time,
    Handled,
};
use std::io::Write;
use mimalloc::MiMalloc;
use clap::Parser;
use std::path::PathBuf;
use ahash::AHashMap;
#[cfg(debug_assertions)]
use libafl::prelude::MultiMonitor;
#[cfg(not(debug_assertions))]
use libafl::prelude::TuiMonitor;

mod input;
mod tokens;
mod mutators;
mod executor;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

const MAP_SIZE: usize = 65536;

#[derive(clap::Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Subcommand,
}

#[derive(clap::Subcommand)]
enum Subcommand {
    Fuzz {
        #[arg(long)]
        output: String,
        
        #[arg(long)]
        libdesock: String,
        
        #[arg(long)]
        corpus: Option<String>,
        
        #[arg(long, default_value_t = String::from("0"))]
        cores: String,
        
        #[arg(long)]
        dict: Option<String>,

        #[arg(long)]
        extra_binary: Vec<String>,
        
        command: Vec<String>,
    },
    
    Print {
        #[arg(long)]
        debug: bool,
        
        input: String
    },
}

fn fuzz(output: String, libdesock: String, corpus: Option<String>, cores: String, dict: Option<String>, mut binaries: Vec<String>, mut command: Vec<String>) -> Result<(), Error> {
    let mut binary_map = AHashMap::new();
    let cores = Cores::from_cmdline(&cores)?;
    let default_binary = command.remove(0);

    /* Map binaries to cores */
    binaries.insert(0, default_binary);

    let mut cursor = 0;
    for core in &cores.ids {
        binary_map.insert(core.0, cursor);
        cursor = (cursor + 1) % binaries.len();
    }
    
    let mut run_client = |state: Option<_>, mut mgr: LlmpRestartingEventManager<_, _, _, _, _>, client: ClientDescription| {
        let this_binary = &binaries[*binary_map.get(&client.core_id().0).unwrap()];

        #[cfg(debug_assertions)]
        println!("Using binary {} on core #{}", this_binary, client.core_id().0);
        
        let seed = current_nanos().rotate_right(client.core_id().0 as u32);
        let mut shmem_provider = UnixShMemProvider::new()?;
        let mut covmap = shmem_provider.new_shmem(MAP_SIZE)?;
        covmap.write_to_env("__AFL_SHM_ID")?;
        std::env::set_var("AFL_MAP_SIZE", format!("{}", MAP_SIZE));

        let edges_observer = unsafe {
            HitcountsMapObserver::new(StdMapObserver::new("edges", covmap.as_slice_mut())).track_indices()
        };
        let edges_handle = edges_observer.handle();
        let time_observer = TimeObserver::new("time");

        let map_feedback = MaxMapFeedback::new(&edges_observer);
        
        let calibration = CalibrationStage::new(&map_feedback);
        
        let mut feedback = feedback_or!(
            map_feedback,
            TimeFeedback::new(&time_observer)
        );
        
        let mut objective = feedback_and_fast!(
            feedback_or!(
                CrashFeedback::new(),
                TimeoutFeedback::new()
            ),
            MaxMapFeedback::with_name("edges_objective", &edges_observer)
        );
        
        let mut state = if let Some(state) = state { 
            state
        } else {
            StdState::new(
                StdRand::with_seed(seed),
                InMemoryOnDiskCorpus::<input::PacketBasedInput<tokens::TokenStream>>::new(format!("{}/queue", output))?,
                OnDiskCorpus::new(format!("{}/crashes", output))?,
                &mut feedback,
                &mut objective,
            )?
        };
        
        if let Some(dict) = &dict {
            state.add_metadata(Tokens::from_file(dict)?);
        }
        
        let max_packets = 16;
        let mutators = tuple_list!(
            mutators::PacketCopyMutator::new(max_packets),
            mutators::PacketDeleteMutator::new(1),
            mutators::PacketRepeatMutator::new(max_packets),
            mutators::PacketSwapMutator::new(),
            mutators::PacketContentMutator::new(mutators::TokenStreamMutator::new(16, seed)),
            mutators::PacketContentMutator::new(mutators::TokenStreamMutator::new(16, seed + 1)),
            mutators::PacketContentMutator::new(mutators::TokenStreamMutator::new(16, seed + 2)),
            mutators::RandomPacketInsertionMutator::new(max_packets),
            mutators::PacketCrossoverMutator::new(max_packets, seed),
            mutators::PacketGenerationMutator::new(max_packets)
        );
        let mutator = StdScheduledMutator::with_max_stack_pow(mutators, 4);
        
        let scheduler = QueueScheduler::new();
        
        let mut fuzzer = StdFuzzer::new(scheduler, feedback, objective);
        
        let mut executor = executor::LibdesockExecutor::new(
            &mut shmem_provider,
            tuple_list!(edges_observer, time_observer),
            &edges_handle,
            this_binary,
            &command,
            5000,
            &libdesock,
        )?;
        
        if state.must_load_initial_inputs() {
            if let Some(corpus) = &corpus {
                state.load_initial_inputs_multicore(
                    &mut fuzzer,
                    &mut executor,
                    &mut mgr, 
                    &[PathBuf::from(corpus)],
                    &client.core_id(),
                    &cores,
                )?;
            }

            state.load_initial_inputs_multicore(
                &mut fuzzer,
                &mut executor,
                &mut mgr,
                &[format!("{}/queue", output).into()],
                &client.core_id(),
                &cores,
            )?;

            if state.corpus().count() == 0 {
                let empty = input::PacketBasedInput::parse_txt(b"").unwrap();
                fuzzer.add_input(
                    &mut state,
                    &mut executor,
                    &mut mgr,
                    empty,
                )?;
            }
        }
        
        let mut stages = tuple_list!(calibration, StdMutationalStage::new(mutator));

        fuzzer.fuzz_loop(&mut stages, &mut executor, &mut state, &mut mgr)?;
        
        Ok(())
    };
    
    #[cfg(debug_assertions)]
    let ui_monitor = MultiMonitor::new(|s| println!("{}", s));
    #[cfg(not(debug_assertions))]
    let ui_monitor = TuiMonitor::builder()
        .title("Libdesock Fuzzer")
        .enhanced_graphics(false)
        .build();
    let mut last_updated = 0;
    let disk_monitor = OnDiskJsonMonitor::new(
        format!("{}/log.jsonl", output),
        move |_| {
            let now = current_time().as_secs();
            
            if (now - last_updated) >= 60 {
                last_updated = now;
                true
            } else {
                false
            }
        },
    );
    
    let shmem_provider = StdShMemProvider::new()?;

    match Launcher::builder()
        .shmem_provider(shmem_provider)
        .configuration(EventConfig::AlwaysUnique)
        .monitor(tuple_list!(ui_monitor, disk_monitor))
        .run_client(&mut run_client)
        .cores(&cores)
        .build()
        .launch()
    {
        Err(Error::ShuttingDown) | Ok(()) => Ok(()),
        e => e,
    }
}

fn print(debug: bool, input: String) -> Result<(), Error> {
    let input = input::PacketBasedInput::<tokens::TokenStream>::from_file(input)?;
    
    if debug {
        println!("{:#?}", input);
    } else {
        let mut buf = vec![0u8; executor::PACKET_BUFFER_SIZE];
        let len = input.convert_to_txt(&mut buf);
        std::io::stdout().write_all(&buf[..len])?;
    }
    
    Ok(())
}

fn main() {
    let args = Args::parse();
    
    match args.command {
        Subcommand::Fuzz { output, libdesock, corpus, cores, dict, extra_binary, command } => fuzz(output, libdesock, corpus, cores, dict, extra_binary, command).expect("Fuzzing failed"),
        Subcommand::Print { debug, input } => print(debug, input).expect("Printing failed"),
    }
}
