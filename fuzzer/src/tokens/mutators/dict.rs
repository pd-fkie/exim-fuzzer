use crate::tokens::{TokenStream, TextToken};
use libafl_bolts::prelude::Rand;
use libafl::prelude::Tokens;

pub fn mutate_dict_insert<R: Rand>(rand: &mut R, stream: &mut TokenStream, dict: &Tokens, max_len: usize) -> bool {
    if dict.is_empty() || stream.len() >= max_len {
        return false;
    }
    
    let cap = max_len - stream.len();
    let idx = rand.between(0, stream.len());
    let item = rand.between(0, dict.len() - 1);
    let new_elem = dict.tokens()[item].to_owned();
    
    if cap >= 3 && rand.coinflip(0.5) {
        let new_elems = [
            TextToken::random_whitespace::<_, 1, 1>(rand),
            TextToken::Constant(new_elem),
            TextToken::random_whitespace::<_, 1, 1>(rand),
        ];
        stream.tokens_mut().splice(idx..idx, new_elems);
    } else {
        stream.tokens_mut().insert(idx, TextToken::Constant(new_elem));
    }
    
    debug_assert!(stream.len() <= max_len);
    true
}

pub fn mutate_dict_replace<R: Rand>(rand: &mut R, stream: &mut TokenStream, dict: &Tokens) -> bool {
    if dict.is_empty() || stream.is_empty() {
        return false;
    }
    
    let start = rand.between(0, stream.len() - 1);
    let item = rand.between(0, dict.len() - 1);
    let new_elem = dict.tokens()[item].to_owned();
    
    for token in &mut stream.tokens_mut()[start..] {
        if !token.is_whitespace() {
            *token = TextToken::Constant(new_elem);
            return true;
        }
    }
    
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::Packet;
    use libafl_bolts::prelude::{StdRand, current_nanos};
    
    #[test]
    fn test_dict_insert() {
        let mut buffer = [0; 1024];
        let mut rand = StdRand::with_seed(current_nanos());
        let stream = "PORT 127,0,0,1,80,80\r\n".parse::<TokenStream>().unwrap();
        let mut dict = Tokens::new();
        dict.add_tokens([
            &b"X".to_vec(),
            &b"Y".to_vec(),
            &b"Z".to_vec(),
        ]);
        
        for _ in 0..10 {
            let mut stream = stream.clone();
            mutate_dict_insert(&mut rand, &mut stream, &dict, 16);
            let size = stream.serialize_content(&mut buffer);
            let s = std::str::from_utf8(&buffer[0..size]).unwrap();
            println!("{}", s);
        }
    }
    
    #[test]
    fn test_dict_replace() {
        let mut buffer = [0; 1024];
        let mut rand = StdRand::with_seed(current_nanos());
        let stream = "PORT 127,0,0,1,80,80\r\n".parse::<TokenStream>().unwrap();
        let mut dict = Tokens::new();
        dict.add_tokens([
            &b"X".to_vec(),
            &b"Y".to_vec(),
            &b"Z".to_vec(),
        ]);
        
        for _ in 0..10 {
            let mut stream = stream.clone();
            mutate_dict_replace(&mut rand, &mut stream, &dict);
            let size = stream.serialize_content(&mut buffer);
            let s = std::str::from_utf8(&buffer[0..size]).unwrap();
            println!("{}", s);
        }
    }
}
