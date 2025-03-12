use libafl::prelude::{
    Forkserver,
    Executor,
    ExitKind,
    Error,
    HasObservers,
    MapObserver,
    ObserversTuple,
    HasExecutions,
};
use libafl_bolts::prelude::{
    RefIndexable,
    UnixShMem,
    UnixShMemProvider,
    ShMemProvider,
    ShMem,
    Truncate,
    MatchNameRef,
    Handle,
};
use std::ops::DerefMut;
use nix::sys::time::TimeSpec;
use nix::unistd::Pid;
use std::marker::PhantomData;
use std::time::Duration;
use crate::input::{PacketBasedInput, Packet};

pub const PACKET_BUFFER_SIZE: usize = 64 * 1024 * 1024;
const ENV_VAR: &str = "LIBDESOCK_PACKET_BUFFER";

#[repr(C)]
struct PacketBuffer {
    cursor: usize,
    size: usize,
    data: [u8; PACKET_BUFFER_SIZE - 16],
}

fn do_forkserver_handshake(forkserver: &mut Forkserver) -> Result<Option<usize>, Error> {
    const FS_NEW_ERROR: i32 = 0xeffe0000_u32 as i32;
    const FS_NEW_OPT_MAPSIZE: i32 = 1_u32 as i32;
    const FS_NEW_OPT_AUTODTCT: i32 = 0x00000800_u32 as i32;
    
    let mut map_size = None;
    let version_status = forkserver.read_st()?;
    
    if (version_status & FS_NEW_ERROR) == FS_NEW_ERROR {
        return Err(Error::unknown("Forkserver instrumentation in target complained".to_string()));
    }
    
    if !(0x41464c00..0x41464cff).contains(&version_status) {
        return Err(Error::unknown("Old forkserver version".to_string()));
    }
    
    let version: u32 = version_status as u32 - 0x41464c00_u32;
    
    if version != 1 {
        return Err(Error::unknown("Forkserver version not supported".to_string()));
    }
    
    forkserver.write_ctl((version_status as u32 ^ 0xffffffff_u32) as i32)?;
    
    let status = forkserver.read_st()?;
    
    if (status & FS_NEW_OPT_MAPSIZE) == FS_NEW_OPT_MAPSIZE {
        map_size = Some(forkserver.read_st()? as usize);
    }
    
    if (status & FS_NEW_OPT_AUTODTCT) != 0 {
        let size = forkserver.read_st()?;
        forkserver.read_st_of_len(size as usize)?;
    }
    
    if forkserver.read_st()? != version_status {
        return Err(Error::unknown("Forkserver end of handshake not caught correctly".to_string()));
    }
    
    Ok(map_size)
}

#[derive(Debug)]
pub struct LibdesockExecutor<P, OT, S> {
    forkserver: Forkserver,
    observers: OT,
    packet_buffer: UnixShMem,
    timeout: TimeSpec,
    phantom: PhantomData<(P, S)>,
}

impl<P, OT, S> LibdesockExecutor<P, OT, S>
where
    P: Packet,
    OT: ObserversTuple<PacketBasedInput<P>, S>
{
    pub fn new<M, T>(shmem_provider: &mut UnixShMemProvider, mut observers: OT, map_observer: &Handle<M>, program: &str, args: &[String], timeout: u64, libdesock_path: &str) -> Result<Self, Error>
    where
        M: AsMut<T>,
        T: MapObserver + Truncate,
    {
        // Construct shared packet buffer
        let packet_buffer = shmem_provider.new_shmem(PACKET_BUFFER_SIZE)?;
        packet_buffer.write_to_env(ENV_VAR)?;
        
        // Construct forkserver
        let mut forkserver = Forkserver::new(
            program.into(),
            args.iter().map(|s| s.into()).collect(),
            vec![
                ("LD_PRELOAD".into(), libdesock_path.into())
            ],
            -1,
            false,
            0,
            false,
            true,
            false,
            Some(crate::MAP_SIZE),
            false,
        )?;
        
        if let Some(map_size) = do_forkserver_handshake(&mut forkserver)? {
            let map_observer: &mut M = observers.get_mut(map_observer).ok_or_else(|| Error::unknown("Map observer not found in observers list"))?;
            map_observer.as_mut().truncate(map_size);
        }
        
        let timeout = Duration::from_millis(timeout);
        
        Ok(Self {
            forkserver,
            observers,
            packet_buffer,
            timeout: timeout.into(),
            phantom: PhantomData,
        })
    }
}

impl<P, OT, S, EM, Z> Executor<EM, PacketBasedInput<P>, S, Z> for LibdesockExecutor<P, OT, S>
where
    S: HasExecutions,
    P: Packet,
    
{
    fn run_target(&mut self, _fuzzer: &mut Z, state: &mut S, _mgr: &mut EM, input: &PacketBasedInput<P>) -> Result<ExitKind, Error> {
        *state.executions_mut() += 1;
        
        // Serialize input into packet buffer
        let packet_buffer = self.packet_buffer.deref_mut().as_mut_ptr().cast::<PacketBuffer>();
        let packet_buffer = unsafe { &mut *packet_buffer };
        packet_buffer.cursor = 0;
        packet_buffer.size = input.convert_to_txt(&mut packet_buffer.data);
        
        // Spin off target
        let mut exit_kind = ExitKind::Ok;
        
        let last_run_timed_out = self.forkserver.last_run_timed_out_raw();
        self.forkserver.set_last_run_timed_out(false);
        self.forkserver.write_ctl(last_run_timed_out)?;
        
        let pid = self.forkserver.read_st()?;
        if pid <= 0 {
            return Err(Error::unknown("Got invalid PID from forkserver".to_string()));
        }
        self.forkserver.set_child_pid(Pid::from_raw(pid));
        
        if let Some(status) = self.forkserver.read_st_timed(&self.timeout)? {
            if libc::WIFSIGNALED(status) {
                exit_kind = ExitKind::Crash;
            }
            if !libc::WIFSTOPPED(status) {
                self.forkserver.reset_child_pid();
            }
        } else {
            self.forkserver.set_last_run_timed_out(true);
            let _ = unsafe { libc::kill(self.forkserver.child_pid().as_raw(), libc::SIGKILL) };
            self.forkserver.read_st()?;
            exit_kind = ExitKind::Timeout;
        }
        
        Ok(exit_kind)
    }
}

impl<P, OT, S> HasObservers for LibdesockExecutor<P, OT, S> {
    type Observers = OT;
    
    fn observers(&self) -> RefIndexable<&Self::Observers, Self::Observers> {
        RefIndexable::from(&self.observers)
    }
    
    fn observers_mut(&mut self) -> RefIndexable<&mut Self::Observers, Self::Observers> {
        RefIndexable::from(&mut self.observers)
    }    
}


