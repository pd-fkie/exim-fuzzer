use crate::tokens::{TokenStream, mutators::common::random_range};
use libafl_bolts::prelude::Rand;

pub fn mutate_copy<R: Rand>(rand: &mut R, stream: &mut TokenStream, max_len: usize) -> bool {
    if stream.is_empty() || stream.len() >= max_len {
        return false;
    }
    
    let range = random_range(rand, stream.len(), max_len - stream.len());
    let new_elems = stream.tokens()[range].to_owned();
    
    let idx = rand.between(0, stream.len());
    stream.tokens_mut().splice(idx..idx, new_elems);
    
    debug_assert!(stream.len() <= max_len);
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::Packet;
    use libafl_bolts::prelude::{StdRand, current_nanos};
    
    #[test]
    fn test_copy() {
        let mut buffer = [0; 1024];
        let mut rand = StdRand::with_seed(current_nanos());
        let stream = "PORT 127,0,0,1,80,80\r\n".parse::<TokenStream>().unwrap();
        
        for _ in 0..10 {
            let mut stream = stream.clone();
            mutate_copy(&mut rand, &mut stream, 16);
            let size = stream.serialize_content(&mut buffer);
            let s = std::str::from_utf8(&buffer[0..size]).unwrap();
            println!("{}", s);
        }
    }
}
