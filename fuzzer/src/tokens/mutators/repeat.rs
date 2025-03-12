use crate::tokens::TokenStream;
use libafl_bolts::prelude::Rand;

pub fn mutate_repeat_token<R: Rand, const AMNT: usize>(rand: &mut R, stream: &mut TokenStream, max_len: usize) -> bool {
    if stream.is_empty() || stream.len() >= max_len {
        return false;
    }
    
    let idx = rand.between(0, stream.len() - 1);
    let elem = &stream.tokens()[idx];
    
    if elem.is_empty() {
        return false;
    }
    
    let elem = elem.clone();
    
    let n = 1 + (AMNT / elem.len());
    let n = std::cmp::min(n, max_len - stream.len());
    
    stream.tokens_mut().splice(idx..idx, vec![elem; n]);
    
    debug_assert!(stream.len() <= max_len);
    true
}

pub fn mutate_repeat_char<R: Rand, const AMNT: usize>(rand: &mut R, stream: &mut TokenStream) -> bool {
    if stream.is_empty() {
        return false;
    }
    
    let idx = rand.between(0, stream.len() - 1);
    let elem = &mut stream.tokens_mut()[idx];
    
    if elem.is_constant() {
        return false;
    }
    
    let elem_len = elem.len();
    
    if elem_len == 0 || elem_len >= AMNT {
        return false;
    }
    
    let n = AMNT - elem_len;
    let idx = rand.between(0, elem_len - 1);
    
    if idx == 0 && elem.is_number() {
        return false;
    }
    
    let c = elem.data()[idx];
    elem.data_mut().splice(idx..idx, vec![c; n]);
    
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::Packet;
    use libafl_bolts::prelude::{StdRand, current_nanos};
    
    #[test]
    fn test_repeat_token() {
        let mut buffer = [0; 1024];
        let mut rand = StdRand::with_seed(current_nanos());
        let stream = "PORT 127,0,0,1,80,80\r\n".parse::<TokenStream>().unwrap();
        
        for _ in 0..10 {
            let mut stream = stream.clone();
            mutate_repeat_token::<_, 16>(&mut rand, &mut stream, 32);
            let size = stream.serialize_content(&mut buffer);
            let s = std::str::from_utf8(&buffer[0..size]).unwrap();
            println!("{}", s);
        }
    }
    
    #[test]
    fn test_repeat_char() {
        let mut buffer = [0; 1024];
        let mut rand = StdRand::with_seed(current_nanos());
        let stream = "PORT 127,0,0,1,80,80\r\n".parse::<TokenStream>().unwrap();
        
        for _ in 0..10 {
            let mut stream = stream.clone();
            mutate_repeat_char::<_, 16>(&mut rand, &mut stream);
            let size = stream.serialize_content(&mut buffer);
            let s = std::str::from_utf8(&buffer[0..size]).unwrap();
            println!("{}", s);
        }
    }
}
