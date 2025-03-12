use crate::tokens::{TokenStream, TextToken};
use libafl_bolts::prelude::Rand;

pub fn mutate_random_insert<R: Rand>(rand: &mut R, stream: &mut TokenStream, max_len: usize) -> bool {
    if stream.len() >= max_len {
        return false;
    }
    
    let idx = rand.between(0, stream.len());
    let new_elem = match rand.between(0, 4) {
        0 => TextToken::random_number::<_, 16>(rand),
        1 => TextToken::random_whitespace::<_, 1, 16>(rand),
        2 ..= 4 => TextToken::random_text::<_, 1, 16>(rand),
        _ => unreachable!(),
    };
    stream.tokens_mut().insert(idx, new_elem);
    
    debug_assert!(stream.len() <= max_len);
    true
}

pub fn mutate_random_replace<R: Rand>(rand: &mut R, stream: &mut TokenStream) -> bool {
    if stream.is_empty() {
        return false;
    }
    
    let idx = rand.between(0, stream.len() - 1);
    let new_elem = match rand.between(0, 4) {
        0 => TextToken::random_number::<_, 16>(rand),
        1 => TextToken::random_whitespace::<_, 1, 16>(rand),
        2 ..= 4 => TextToken::random_text::<_, 1, 16>(rand),
        _ => unreachable!(),
    };
    stream.tokens_mut()[idx] = new_elem;
    
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::Packet;
    use libafl_bolts::prelude::{StdRand, current_nanos};
    
    #[test]
    fn test_random_insert() {
        let mut buffer = [0; 1024];
        let mut rand = StdRand::with_seed(current_nanos());
        let stream = "PORT 127,0,0,1,80,80\r\n".parse::<TokenStream>().unwrap();
        
        for _ in 0..10 {
            let mut stream = stream.clone();
            mutate_random_insert(&mut rand, &mut stream, 16);
            let size = stream.serialize_content(&mut buffer);
            let s = std::str::from_utf8(&buffer[0..size]).unwrap();
            println!("{}", s);
        }
    }
    
    #[test]
    fn test_random_replace() {
        let mut buffer = [0; 1024];
        let mut rand = StdRand::with_seed(current_nanos());
        let stream = "PORT 127,0,0,1,80,80\r\n".parse::<TokenStream>().unwrap();
        
        for _ in 0..10 {
            let mut stream = stream.clone();
            mutate_random_replace(&mut rand, &mut stream);
            let size = stream.serialize_content(&mut buffer);
            let s = std::str::from_utf8(&buffer[0..size]).unwrap();
            println!("{}", s);
        }
    }
}
