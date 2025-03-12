use crate::tokens::{TokenStream, TextToken};
use libafl_bolts::prelude::Rand;

const WHITESPACE: [u8; 6] = [b' ', b'\t', b'\n', 0x0b, 0x0c, b'\r'];
const DIGITS: [u8; 10] = [b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9'];
const TEXT_ALLOW_MAP: [bool; 256] = [true, true, true, true, true, true, true, true, true, false, false, false, false, false, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, false, true, true, true, true, true, true, true, true, true, true, false, true, false, true, true, false, false, false, false, false, false, false, false, false, false, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false];

#[inline]
fn find_valid_byte(mut c: u8) -> u8 {
    while !TEXT_ALLOW_MAP[c as usize] {
        c = c.wrapping_add(1);
    }
    
    c
}

pub fn mutate_flip<R: Rand>(rand: &mut R, stream: &mut TokenStream) -> bool {
    if stream.is_empty() {
        return false;
    }
    
    let idx = rand.between(0, stream.len() - 1);
    
    match &mut stream.tokens_mut()[idx] {
        TextToken::Constant(_) => return false,
        TextToken::Whitespace(data) => {
            if data.is_empty() {
                return false;
            }
            
            let idx = rand.between(0, data.len() - 1);
            data[idx] = rand.choose(WHITESPACE).unwrap();
        },
        TextToken::Number(data) => {
            if data.is_empty() {
                return false;
            }
            
            let idx = rand.between(0, data.len() - 1);
            
            if idx == 0 && data.len() > 1 {
                match rand.between(0, 2) {
                    0 => data[idx] = b'-',
                    1 => data[idx] = b'+',
                    2 => data[idx] = rand.choose(DIGITS).unwrap(),
                    _ => unreachable!(),
                }
            } else {
                data[idx] = rand.choose(DIGITS).unwrap();
            }
        },
        TextToken::Text(data) => {
            if data.is_empty() {
                return false;
            }
            
            let idx = rand.between(0, data.len() - 1);
            let new_value = (rand.next() as u8) & 0x7F;
            data[idx] = find_valid_byte(new_value);
        },
    }
    
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::Packet;
    use libafl_bolts::prelude::{StdRand, current_nanos};
    
    #[test]
    fn test_flip() {
        let mut buffer = [0; 1024];
        let mut rand = StdRand::with_seed(current_nanos());
        let stream = "PORT 127,0,0,1,80,80\r\n".parse::<TokenStream>().unwrap();
        
        for _ in 0..10 {
            let mut stream = stream.clone();
            mutate_flip(&mut rand, &mut stream);
            let size = stream.serialize_content(&mut buffer);
            let s = std::str::from_utf8(&buffer[0..size]).unwrap();
            println!("{}", s);
        }
    }
}
