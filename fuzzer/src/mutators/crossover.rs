use libafl_bolts::prelude::{Named, Rand, StdRand};
use libafl::prelude::{Mutator, MutationResult, Error, HasRand, HasCorpus, random_corpus_id, Corpus};
use crate::input::{PacketBasedInput, Packet};
use std::borrow::Cow;

pub struct PacketCrossoverMutator {
    max_length: usize,
    rand: StdRand,
}

impl PacketCrossoverMutator {
    #[allow(clippy::new_without_default)]
    pub fn new(max_length: usize, seed: u64) -> Self {
        Self {
            max_length,
            rand: StdRand::with_seed(seed),
        }
    }
}

impl Named for PacketCrossoverMutator {
    fn name(&self) -> &Cow<'static, str> {
        static NAME: Cow<'static, str> = Cow::Borrowed("PacketCrossoverMutator");
        &NAME
    }
}

impl<P, S> Mutator<PacketBasedInput<P>, S> for PacketCrossoverMutator
where
    P: Packet + Clone,
    S: HasRand + HasCorpus<PacketBasedInput<P>>,
{
    fn mutate(&mut self, state: &mut S, input: &mut PacketBasedInput<P>) -> Result<MutationResult, Error> {
        if input.packets().len() >= self.max_length {
            return Ok(MutationResult::Skipped);
        }
        
        let idx = random_corpus_id!(state.corpus(), &mut self.rand);
                    
        if state.corpus().current().as_ref() == Some(&idx) {
            return Ok(MutationResult::Skipped);
        }
        
        let mut other_testcase = state.corpus().get(idx)?.borrow_mut();
        let other_testcase = other_testcase.load_input(state.corpus())?;
        
        if other_testcase.packets().is_empty() {
            return Ok(MutationResult::Skipped);
        }
        
        let idx = self.rand.between(0, other_testcase.packets().len() - 1);
        let other_packet = &other_testcase.packets()[idx];
        
        let idx = self.rand.between(0, input.packets().len());
        input.packets_mut().insert(idx, other_packet.to_owned());
        
        Ok(MutationResult::Mutated)
    }
}
