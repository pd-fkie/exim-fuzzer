use crate::tokens::{TokenStream, TextToken, mutators::common::copy_vec};
use libafl_bolts::prelude::Rand;
use libafl::prelude::Tokens;
use smallvec::SmallVec;
use std::ops::Range;

pub fn mutate_swap_tokens<R: Rand>(rand: &mut R, stream: &mut TokenStream) -> bool {
    if stream.is_empty() {
        return false;
    }
    
    let from = rand.between(0, stream.len() - 1);
    let to = rand.between(0, stream.len() - 1);
    
    if from == to {
        return false;
    }
    
    stream.tokens_mut().swap(from, to);
    
    true
}

pub fn mutate_swap_constants<R: Rand>(rand: &mut R, stream: &mut TokenStream, dict: &Tokens) -> bool {
    if stream.is_empty() || dict.is_empty() {
        return false;
    }
    
    let start = rand.between(0, stream.len() - 1);
    
    for token in &mut stream.tokens_mut()[start..] {
        if let TextToken::Constant(data) = token {
            let idx = rand.between(0, dict.len() - 1);
            copy_vec(data, &dict.tokens()[idx]);
            return true;
        }
    }
    
    false
}

#[inline]
fn range_dist(a: &Range<usize>, b: &Range<usize>) -> usize {
    let a_len = a.end - a.start;
    let b_len = b.end - b.start;
    a_len.wrapping_sub(b_len)
}

pub fn mutate_swap_words<R: Rand>(rand: &mut R, stream: &mut TokenStream) -> bool {
    if stream.len() < 3 {
        return false;
    }
    
    /* Collect words */
    let mut words = SmallVec::<[Range<usize>; 32]>::new();
    let mut start = 0;
    
    for (i, token) in stream.tokens().iter().enumerate() {
        if token.is_whitespace() {
            words.push(Range {
                start,
                end: i,
            });
            start = i + 1;
        }
    }
    
    words.push(Range {
        start,
        end: stream.len(),
    });
    
    /* Select words */
    let mut from = rand.between(0, words.len() - 1);
    let mut to = rand.between(0, words.len() - 1);
    
    if from == to {
        return false;
    }
    
    /* Let the heebie jeebies commence */
    if from > to {
        std::mem::swap(&mut from, &mut to);
    }
    
    let to_slice = stream.tokens_mut().splice(words[to].clone(), []).collect::<Vec<_>>();
    let from_slice = stream.tokens_mut().splice(words[from].clone(), to_slice).collect::<Vec<_>>();
    
    let delta = range_dist(&words[to], &words[from]);
    let idx = words[to].start.wrapping_add(delta);
    
    stream.tokens_mut().splice(idx..idx, from_slice);
    
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::Packet;
    use libafl_bolts::prelude::{StdRand, current_nanos};
    
    #[test]
    fn test_swap_tokens() {
        let mut buffer = [0; 1024];
        let mut rand = StdRand::with_seed(current_nanos());
        let stream = "PORT 127,0,0,1,80,80\r\n".parse::<TokenStream>().unwrap();
        
        for _ in 0..10 {
            let mut stream = stream.clone();
            mutate_swap_tokens(&mut rand, &mut stream);
            let size = stream.serialize_content(&mut buffer);
            let s = std::str::from_utf8(&buffer[0..size]).unwrap();
            println!("{}", s);
        }
    }
    
    #[test]
    fn test_swap_constants() {
        let mut buffer = [0; 1024];
        let mut rand = StdRand::with_seed(current_nanos());
        let mut stream = " 127,0,0,1,80,80\r\n".parse::<TokenStream>().unwrap();
        stream.tokens_mut().insert(0, TextToken::Constant(b"PORT".to_vec()));
        let mut dict = Tokens::new();
        dict.add_tokens([
            &b"X".to_vec(),
            &b"Y".to_vec(),
            &b"Z".to_vec(),
        ]);
        
        for _ in 0..10 {
            let mut stream = stream.clone();
            mutate_swap_constants(&mut rand, &mut stream, &dict);
            let size = stream.serialize_content(&mut buffer);
            let s = std::str::from_utf8(&buffer[0..size]).unwrap();
            println!("{}", s);
        }
    }
    
    #[test]
    fn test_swap_words() {
        let mut buffer = [0; 1024];
        let mut rand = StdRand::with_seed(current_nanos());
        let stream = "1 2 3 4 5 6 7 8 9".parse::<TokenStream>().unwrap();
        
        for _ in 0..10 {
            let mut stream = stream.clone();
            mutate_swap_words(&mut rand, &mut stream);
            let size = stream.serialize_content(&mut buffer);
            let s = std::str::from_utf8(&buffer[0..size]).unwrap();
            println!("{}", s);
        }
    }
}
