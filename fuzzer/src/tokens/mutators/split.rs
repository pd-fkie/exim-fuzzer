use crate::tokens::{TokenStream, TextToken};
use libafl_bolts::prelude::Rand;

pub fn mutate_split<R: Rand>(rand: &mut R, stream: &mut TokenStream, max_len: usize) -> bool {
    if stream.is_empty() || max_len.saturating_sub(stream.len()) < 2 {
        return false;
    }
    
    let idx = rand.between(0, stream.len() - 1);
    let token = &mut stream.tokens_mut()[idx];
    
    if token.len() <= 1 || token.is_constant() {
        return false;
    }
    
    let pos = 1 + rand.between(0, token.len() - 2);
    
    if token.is_number() && pos == 1 {
        return false;
    }
    
    let mut split_elem = token.clone_nodata();
    *split_elem.data_mut() = token.data_mut().split_off(pos);
    
    let new_elem = match rand.between(0, 3) {
        0 => TextToken::random_number::<_, 16>(rand),
        1 => TextToken::random_whitespace::<_, 1, 16>(rand),
        2 ..= 3 => TextToken::random_text::<_, 1, 16>(rand),
        _ => unreachable!(),
    };
    
    stream.tokens_mut().splice(idx + 1..idx + 1, [new_elem, split_elem]);
    
    debug_assert!(stream.len() <= max_len);
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::Packet;
    use libafl_bolts::prelude::{StdRand, current_nanos};
    
    #[test]
    fn test_split() {
        let mut buffer = [0; 1024];
        let mut rand = StdRand::with_seed(current_nanos());
        let stream = "200 fuck my shit up".parse::<TokenStream>().unwrap();
        let mut count = 0;
        
        for _ in 0..10 {
            let mut stream = stream.clone();
            
            if mutate_split(&mut rand, &mut stream, 16) {
                let size = stream.serialize_content(&mut buffer);
                let s = std::str::from_utf8(&buffer[0..size]).unwrap();
                println!("{}", s);
                count += 1;
            }
        }
        
        println!();
        println!("Mutated {}/10", count);
    }
}
