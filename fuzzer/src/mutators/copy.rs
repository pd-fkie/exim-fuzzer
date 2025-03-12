use libafl_bolts::prelude::{Named, Rand};
use libafl::prelude::{Mutator, MutationResult, Error, HasRand};
use crate::input::{PacketBasedInput, Packet};
use std::borrow::Cow;

pub struct PacketCopyMutator {
    max_length: usize,
}

impl PacketCopyMutator {
    #[allow(clippy::new_without_default)]
    pub fn new(max_length: usize) -> Self {
        Self {
            max_length,
        }
    }
}

impl Named for PacketCopyMutator {
    fn name(&self) -> &Cow<'static, str> {
        static NAME: Cow<'static, str> = Cow::Borrowed("PacketCopyMutator");
        &NAME
    }
}

impl<P, S> Mutator<PacketBasedInput<P>, S> for PacketCopyMutator
where
    P: Packet + Clone,
    S: HasRand,
{
    fn mutate(&mut self, state: &mut S, input: &mut PacketBasedInput<P>) -> Result<MutationResult, Error> {
        let len = input.packets().len();
        
        if len == 0 || len >= self.max_length {
            return Ok(MutationResult::Skipped);
        }
        
        let to = state.rand_mut().between(0, len);
        let from = state.rand_mut().between(0, len - 1);
        
        let packet = input.packets()[from].clone();
        input.packets_mut().insert(to, packet);
        
        Ok(MutationResult::Mutated)
    }
}
