use libafl_bolts::prelude::{Named, Rand};
use libafl::prelude::{Mutator, MutationResult, Error, HasRand};
use crate::input::{PacketBasedInput, Packet};
use std::borrow::Cow;

pub struct PacketRepeatMutator {
    max_length: usize,
}

impl PacketRepeatMutator {
    #[allow(clippy::new_without_default)]
    pub fn new(max_length: usize) -> Self {
        Self {
            max_length,
        }
    }
}

impl Named for PacketRepeatMutator {
    fn name(&self) -> &Cow<'static, str> {
        static NAME: Cow<'static, str> = Cow::Borrowed("PacketRepeatMutator");
        &NAME
    }
}

impl<P, S> Mutator<PacketBasedInput<P>, S> for PacketRepeatMutator
where
    P: Packet + Clone,
    S: HasRand,
{
    fn mutate(&mut self, state: &mut S, input: &mut PacketBasedInput<P>) -> Result<MutationResult, Error> {
        let len = input.packets().len();
        
        if len == 0 || len >= self.max_length {
            return Ok(MutationResult::Skipped);
        }
        
        let idx = state.rand_mut().between(0, len - 1);
        let n = 1 + state.rand_mut().between(0, (self.max_length - len).saturating_sub(1));
        let packet = input.packets()[idx].clone();
        input.packets_mut().splice(idx..idx, vec![packet; n]);
        
        Ok(MutationResult::Mutated)
    }
}
