use libafl_bolts::prelude::{Named, Rand};
use libafl::prelude::{Mutator, MutationResult, Error, HasRand};
use crate::input::{PacketBasedInput, Packet};
use std::marker::PhantomData;
use std::borrow::Cow;

pub trait PacketMutator<P, S>
where
    P: Packet,
{
    fn mutate_packet(&mut self, state: &mut S, packet: &mut P) -> Result<MutationResult, Error>;
}

pub struct PacketContentMutator<P, S, M>
where
    M: PacketMutator<P, S>,
    P: Packet,
{
    mutator: M,
    phantom: PhantomData<(P, S)>,
}

impl<P, S, M> PacketContentMutator<P, S, M>
where
    M: PacketMutator<P, S>,
    P: Packet,
{
    #[allow(clippy::new_without_default)]
    pub fn new(mutator: M) -> Self {
        Self {
            mutator,
            phantom: PhantomData,
        }
    }
}

impl<P, S, M> Named for PacketContentMutator<P, S, M>
where
    M: PacketMutator<P, S>,
    P: Packet,
{
    fn name(&self) -> &Cow<'static, str> {
        static NAME: Cow<'static, str> = Cow::Borrowed("PacketContentMutator");
        &NAME
    }
}

impl<P, S, M> Mutator<PacketBasedInput<P>, S> for PacketContentMutator<P, S, M>
where
    M: PacketMutator<P, S>,
    P: Packet,
    S: HasRand,
{
    fn mutate(&mut self, state: &mut S, input: &mut PacketBasedInput<P>) -> Result<MutationResult, Error> {
        let len = input.packets().len();
        
        if len == 0 {
            return Ok(MutationResult::Skipped);
        }
        
        let idx = state.rand_mut().between(0, len - 1);
        let packet = &mut input.packets_mut()[idx];
        self.mutator.mutate_packet(state, packet)
    }
}
