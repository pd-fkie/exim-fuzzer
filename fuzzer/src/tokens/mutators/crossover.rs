use crate::tokens::{TokenStream, mutators::common::random_range};
use libafl_bolts::prelude::Rand;

pub fn mutate_crossover_replace<R: Rand>(rand: &mut R, stream: &mut TokenStream, other: &TokenStream, max_len: usize) -> bool {
    if stream.is_empty() || other.is_empty() || stream.len() >= max_len {
        return false;
    }
    
    let dst_range = random_range(rand, stream.len(), stream.len());
    let src_range = random_range(rand, other.len(), max_len - stream.len() + dst_range.end - dst_range.start);
    
    stream.tokens_mut().splice(dst_range, other.tokens()[src_range].to_owned());
    
    debug_assert!(stream.len() <= max_len);
    true
}

pub fn mutate_crossover_insert<R: Rand>(rand: &mut R, stream: &mut TokenStream, other: &TokenStream, max_len: usize) -> bool {
    if stream.is_empty() || other.is_empty() || stream.len() >= max_len {
        return false;
    }
    
    let dst_index = rand.between(0, stream.len());
    let src_range = random_range(rand, other.len(), max_len - stream.len());
    
    stream.tokens_mut().splice(dst_index..dst_index, other.tokens()[src_range].to_owned());
    
    debug_assert!(stream.len() <= max_len);
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::Packet;
    use libafl_bolts::prelude::{StdRand, current_nanos};
    
    #[test]
    fn test_replace() {
        let mut buffer = [0; 1024];
        let mut rand = StdRand::with_seed(current_nanos());
        let stream1 = "200 fuck my shit up".parse::<TokenStream>().unwrap();
        let stream2 = "PORT 127,0,0,1,80,80\r\n".parse::<TokenStream>().unwrap();
        
        for _ in 0..10 {
            let mut stream = stream1.clone();
            mutate_crossover_replace(&mut rand, &mut stream, &stream2, 16);
            let size = stream.serialize_content(&mut buffer);
            let s = std::str::from_utf8(&buffer[0..size]).unwrap();
            println!("{}", s);
        }
    }
    
    #[test]
    fn test_insert() {
        let mut buffer = [0; 1024];
        let mut rand = StdRand::with_seed(current_nanos());
        let stream1 = "200 fuck my shit up".parse::<TokenStream>().unwrap();
        let stream2 = "PORT 127,0,0,1,80,80\r\n".parse::<TokenStream>().unwrap();
        
        for _ in 0..10 {
            let mut stream = stream1.clone();
            mutate_crossover_insert(&mut rand, &mut stream, &stream2, 16);
            let size = stream.serialize_content(&mut buffer);
            let s = std::str::from_utf8(&buffer[0..size]).unwrap();
            println!("{}", s);
        }
    }
}
