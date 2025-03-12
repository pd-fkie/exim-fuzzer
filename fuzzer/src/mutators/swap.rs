use libafl_bolts::prelude::{Named, Rand};
use libafl::prelude::{Mutator, MutationResult, Error, HasRand};
use crate::input::{PacketBasedInput, Packet};
use std::borrow::Cow;

pub struct PacketSwapMutator;

impl PacketSwapMutator {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {}
    }
}

impl Named for PacketSwapMutator {
    fn name(&self) -> &Cow<'static, str> {
        static NAME: Cow<'static, str> = Cow::Borrowed("PacketSwapMutator");
        &NAME
    }
}

impl<P, S> Mutator<PacketBasedInput<P>, S> for PacketSwapMutator
where
    P: Packet,
    S: HasRand,
{
    fn mutate(&mut self, state: &mut S, input: &mut PacketBasedInput<P>) -> Result<MutationResult, Error> {
        let len = input.packets().len();
        
        if len <= 1 {
            return Ok(MutationResult::Skipped);
        }
        
        let to = state.rand_mut().between(0, len - 1);
        let from = state.rand_mut().between(0, len - 1);
        
        if to == from {
            return Ok(MutationResult::Skipped);
        }
        
        input.packets_mut().swap(to, from);
        
        Ok(MutationResult::Mutated)
    }
}
