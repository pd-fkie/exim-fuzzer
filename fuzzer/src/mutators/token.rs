use libafl_bolts::prelude::{Rand, StdRand};
use libafl::prelude::{MutationResult, Error, HasRand, HasMetadata, Tokens, HasCorpus, random_corpus_id, Corpus};
use crate::{
    input::PacketBasedInput,
    tokens::{TokenStream, mutators::*},
    mutators::content::PacketMutator,
};

const STACKS: [usize; 4] = [
    2,
    4,
    8,
    32,
];

pub struct TokenStreamMutator {
    max_tokens: usize,
    rand: StdRand,
}

impl TokenStreamMutator {
    pub fn new(max_tokens: usize, seed: u64) -> Self {
        Self {
            max_tokens,
            rand: StdRand::with_seed(seed),
        }
    }
}

impl<S> PacketMutator<TokenStream, S> for TokenStreamMutator
where
    S: HasRand + HasMetadata + HasCorpus<PacketBasedInput<TokenStream>>,
{
    fn mutate_packet(&mut self, state: &mut S, packet: &mut TokenStream) -> Result<MutationResult, Error> {
        let stack = state.rand_mut().choose(STACKS).unwrap();
        let mut mutated = false;
        
        for _ in 0..stack {
            mutated |= match self.rand.between(0, 18) {
                0 => mutate_copy(&mut self.rand, packet, self.max_tokens),
                1 => {
                    let idx = random_corpus_id!(state.corpus(), &mut self.rand);
                    
                    if state.corpus().current().as_ref() == Some(&idx) {
                        continue;
                    }
                    
                    let mut other_testcase = state.corpus().get(idx)?.borrow_mut();
                    let other_testcase = other_testcase.load_input(state.corpus())?;
                    
                    if other_testcase.packets().is_empty() {
                        continue;
                    }
                    
                    let idx = self.rand.between(0, other_testcase.packets().len() - 1);
                    let other_packet = &other_testcase.packets()[idx];
                    
                    mutate_crossover_insert(&mut self.rand, packet, other_packet, self.max_tokens)
                },
                2 => {
                    let idx = random_corpus_id!(state.corpus(), &mut self.rand);
                    
                    if state.corpus().current().as_ref() == Some(&idx) {
                        continue;
                    }
                    
                    let mut other_testcase = state.corpus().get(idx)?.borrow_mut();
                    let other_testcase = other_testcase.load_input(state.corpus())?;
                    
                    if other_testcase.packets().is_empty() {
                        continue;
                    }
                    
                    let idx = self.rand.between(0, other_testcase.packets().len() - 1);
                    let other_packet = &other_testcase.packets()[idx];
                    
                    mutate_crossover_replace(&mut self.rand, packet, other_packet, self.max_tokens)
                },
                3 => mutate_delete(&mut self.rand, packet),
                4 => mutate_flip(&mut self.rand, packet),
                5 => mutate_interesting(&mut self.rand, packet),
                6 => mutate_random_insert(&mut self.rand, packet, self.max_tokens),
                7 => mutate_random_replace(&mut self.rand, packet),
                8 => mutate_repeat_char::<_, 16>(&mut self.rand, packet),
                9 => mutate_repeat_token::<_, 4>(&mut self.rand, packet, self.max_tokens),
                10 => mutate_special_insert(&mut self.rand, packet),
                11 => mutate_special_replace(&mut self.rand, packet),
                12 => mutate_split(&mut self.rand, packet, self.max_tokens),
                13 => mutate_swap_tokens(&mut self.rand, packet),
                14 => mutate_swap_words(&mut self.rand, packet),
                15 => mutate_truncate(&mut self.rand, packet),
                16 => {
                    let dict = state.metadata_map().get::<Tokens>();
        
                    if let Some(dict) = dict {
                        mutate_dict_insert(&mut self.rand, packet, dict, self.max_tokens)
                    } else {
                        false
                    }
                },
                17 => {
                    let dict = state.metadata_map().get::<Tokens>();
        
                    if let Some(dict) = dict {
                        mutate_dict_replace(&mut self.rand, packet, dict)
                    } else {
                        false
                    }
                },
                18 => {
                    let dict = state.metadata_map().get::<Tokens>();
        
                    if let Some(dict) = dict {
                        mutate_swap_constants(&mut self.rand, packet, dict)
                    } else {
                        false
                    }
                },
                _ => unreachable!(),
            };
        }
        
        if mutated {
            Ok(MutationResult::Mutated)
        } else {
            Ok(MutationResult::Skipped)
        }
    }
}
