use libafl_bolts::prelude::{Named, Rand};
use libafl::prelude::{Mutator, MutationResult, Error, HasRand};
use crate::input::{PacketBasedInput, Packet};
use std::marker::PhantomData;
use std::borrow::Cow;

pub trait RandomPacketCreator<S>
where
    Self: Sized,
{
    fn create_random_packet(state: &mut S) -> Self;
}

pub struct RandomPacketInsertionMutator<P, S>
where
    P: Packet + RandomPacketCreator<S>,
{
    max_packets: usize,
    phantom: PhantomData<(P, S)>,
}

impl<P, S> RandomPacketInsertionMutator<P, S>
where
    P: Packet + RandomPacketCreator<S>,
{
    #[allow(clippy::new_without_default)]
    pub fn new(max_packets: usize) -> Self {
        Self {
            max_packets,
            phantom: PhantomData,
        }
    }
}

impl<P, S> Named for RandomPacketInsertionMutator<P, S>
where
    P: Packet + RandomPacketCreator<S>,
{
    fn name(&self) -> &Cow<'static, str> {
        static NAME: Cow<'static, str> = Cow::Borrowed("RandomPacketInsertionMutator");
        &NAME
    }
}

impl<P, S> Mutator<PacketBasedInput<P>, S> for RandomPacketInsertionMutator<P, S>
where
    P: Packet + RandomPacketCreator<S>,
    S: HasRand,
{
    fn mutate(&mut self, state: &mut S, input: &mut PacketBasedInput<P>) -> Result<MutationResult, Error> {
        let len = input.packets().len();

        if len >= self.max_packets {
            return Ok(MutationResult::Skipped);
        }
        
        let idx = state.rand_mut().between(0, len);
        let new_packet = P::create_random_packet(state);
        input.packets_mut().insert(idx, new_packet);
        Ok(MutationResult::Mutated)
    }
}
