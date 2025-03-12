use libafl_bolts::prelude::{Named, Rand};
use libafl::prelude::{Mutator, MutationResult, Error, HasRand};
use crate::input::{PacketBasedInput, Packet};
use std::marker::PhantomData;
use std::borrow::Cow;

pub trait PacketGenerator<S>
where
    Self: Sized,
{
    fn generate_packets(state: &mut S) -> Vec<Self>;
}

pub struct PacketGenerationMutator<P, S>
where
    P: Packet + PacketGenerator<S>,
{
    max_packets: usize,
    phantom: PhantomData<(P, S)>,
}

impl<P, S> PacketGenerationMutator<P, S>
where
    P: Packet + PacketGenerator<S>,
{
    #[allow(clippy::new_without_default)]
    pub fn new(max_packets: usize) -> Self {
        Self {
            max_packets,
            phantom: PhantomData,
        }
    }
}

impl<P, S> Named for PacketGenerationMutator<P, S>
where
    P: Packet + PacketGenerator<S>,
{
    fn name(&self) -> &Cow<'static, str> {
        static NAME: Cow<'static, str> = Cow::Borrowed("PacketGenerationMutator");
        &NAME
    }
}

impl<P, S> Mutator<PacketBasedInput<P>, S> for PacketGenerationMutator<P, S>
where
    P: Packet + PacketGenerator<S>,
    S: HasRand,
{
    fn mutate(&mut self, state: &mut S, input: &mut PacketBasedInput<P>) -> Result<MutationResult, Error> {
        let len = input.packets().len();

        if len >= self.max_packets {
            return Ok(MutationResult::Skipped);
        }
        
        let idx = state.rand_mut().between(0, len);
        let new_packets = P::generate_packets(state);
        input.packets_mut().splice(idx..idx, new_packets);
        Ok(MutationResult::Mutated)
    }
}
