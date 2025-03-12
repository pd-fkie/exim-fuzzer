use libafl::prelude::{Input, BytesInput, Error};
use libafl_bolts::prelude::HasLen;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::io::Read;
use std::hash::Hash;

const LIBDESOCK_SEPARATOR: &[u8; 8] = b"--------";

pub trait Packet: Sized + Hash {
    fn serialize_content(&self, buffer: &mut [u8]) -> usize;
    fn deserialize_content(buffer: &[u8]) -> Option<Self>;
}

impl Packet for BytesInput {
    fn serialize_content(&self, buffer: &mut [u8]) -> usize {
        let len = std::cmp::min(buffer.len(), self.as_ref().len());
        buffer[..len].copy_from_slice(&self.as_ref()[..len]);
        len
    }
    
    fn deserialize_content(buffer: &[u8]) -> Option<Self> {
        Some(Self::from(buffer))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default, Hash)]
#[serde(bound = "P: Serialize + for<'a> Deserialize<'a>")]
pub struct PacketBasedInput<P>
where
    P: Packet,
{
    packets: Vec<P>,
}

impl<P> PacketBasedInput<P>
where
    P: Packet,
{
    pub fn packets(&self) -> &[P] {
        &self.packets
    }

    pub fn packets_mut(&mut self) -> &mut Vec<P> {
        &mut self.packets
    }
    
    pub fn convert_to_txt(&self, buf: &mut [u8]) -> usize {
        let mut cursor = 0;
        for packet in &self.packets {
            cursor += packet.serialize_content(&mut buf[cursor..]);
            buf[cursor..cursor + LIBDESOCK_SEPARATOR.len()].copy_from_slice(LIBDESOCK_SEPARATOR);
            cursor += LIBDESOCK_SEPARATOR.len();
        }
        cursor.saturating_sub(LIBDESOCK_SEPARATOR.len())
    }
    
    pub fn parse_txt(buf: &[u8]) -> Option<Self> {
        let mut start = 0;
        let mut cursor = 0;
        let mut packets = vec![];
        
        while cursor < buf.len() {
            if cursor + LIBDESOCK_SEPARATOR.len() > buf.len() {
                break;
            }
            
            if &buf[cursor..cursor + LIBDESOCK_SEPARATOR.len()] == LIBDESOCK_SEPARATOR {
                let data = &buf[start..cursor];
                let packet = P::deserialize_content(data)?;
                packets.push(packet);
                
                cursor += LIBDESOCK_SEPARATOR.len();
                start = cursor;
            } else {
                cursor += 1;
            }
        }
        
        if cursor < buf.len() {
            let data = &buf[cursor..];
            let packet = P::deserialize_content(data)?;
            packets.push(packet);
        }
        
        Some(Self {
            packets,
        })
    }
}

impl<P> Input for PacketBasedInput<P>
where
    P: Packet + std::fmt::Debug + Serialize + for<'a> Deserialize<'a> + Clone,
{
    fn from_file<F>(path: F) -> Result<Self, Error>
    where
        F: AsRef<Path>,
    {
        let path = path.as_ref();
        let mut file = std::fs::File::open(path)?;
        let mut bytes = vec![];
        file.read_to_end(&mut bytes)?;
        
        match path.extension().and_then(|s| s.to_str()) {
            Some("txt") => {
                Self::parse_txt(&bytes).ok_or_else(|| Error::serialize(format!("Could not parse txt file {}", path.display())))
            },
            _ => Ok(postcard::from_bytes(&bytes)?),
        }
    }
}

impl<P> HasLen for PacketBasedInput<P>
where
    P: Packet,
{
    fn len(&self) -> usize {
        self.packets.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::TokenStream;
    
    #[test]
    fn test_txt() {
        let input = PacketBasedInput::<TokenStream>::parse_txt(b"----------------abc").unwrap();
        println!("{:?}", input);
        
        let mut buf = vec![0u8; 1024];
        let size = input.convert_to_txt(&mut buf);
        println!("{}", std::str::from_utf8(&buf[..size]).unwrap());
    }
}
