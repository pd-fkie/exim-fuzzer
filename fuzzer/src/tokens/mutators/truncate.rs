use crate::tokens::TokenStream;
use libafl_bolts::prelude::Rand;

pub fn mutate_truncate<R: Rand>(rand: &mut R, stream: &mut TokenStream) -> bool {
    if stream.is_empty() {
        return false;
    }
    
    let idx = rand.between(0, stream.len() - 1);
    let elem = &mut stream.tokens_mut()[idx];
    
    if elem.is_constant() || elem.len() < 2 {
        return false;
    }
    
    let new_len = 1 + rand.between(0, elem.len() - 2);
    
    if new_len == 1 && elem.is_number() && matches!(elem.data().first(), Some(b'-') | Some(b'+')) {
        return false;
    }
     
    elem.data_mut().truncate(new_len);
    
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::Packet;
    use libafl_bolts::prelude::{StdRand, current_nanos};
    
    #[test]
    fn test_delete() {
        let mut buffer = [0; 1024];
        let mut rand = StdRand::with_seed(current_nanos());
        let stream = "here are a couple of words for you".parse::<TokenStream>().unwrap();
        
        for _ in 0..10 {
            let mut stream = stream.clone();
            mutate_truncate(&mut rand, &mut stream);
            let size = stream.serialize_content(&mut buffer);
            let s = std::str::from_utf8(&buffer[0..size]).unwrap();
            println!("{}", s);
        }
    }
}
