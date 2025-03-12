use libafl_bolts::prelude::{Named, Rand};
use libafl::prelude::{Mutator, MutationResult, Error, HasRand};
use crate::input::{PacketBasedInput, Packet};
use std::borrow::Cow;

pub struct PacketDeleteMutator {
    min_length: usize,
}

impl PacketDeleteMutator {
    #[allow(clippy::new_without_default)]
    pub fn new(min_length: usize) -> Self {
        Self {
            min_length,
        }
    }
}

impl Named for PacketDeleteMutator {
    fn name(&self) -> &Cow<'static, str> {
        static NAME: Cow<'static, str> = Cow::Borrowed("PacketDeleteMutator");
        &NAME
    }
}

impl<P, S> Mutator<PacketBasedInput<P>, S> for PacketDeleteMutator
where
    P: Packet,
    S: HasRand,
{
    fn mutate(&mut self, state: &mut S, input: &mut PacketBasedInput<P>) -> Result<MutationResult, Error> {
        let len = input.packets().len();
        
        if len == 0 || len <= self.min_length {
            return Ok(MutationResult::Skipped);
        }
        
        let idx = state.rand_mut().between(0, len - 1);
        input.packets_mut().remove(idx);
        
        Ok(MutationResult::Mutated)
    }
}
