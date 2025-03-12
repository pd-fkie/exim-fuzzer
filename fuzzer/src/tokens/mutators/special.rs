use crate::tokens::{TokenStream, TextToken};
use libafl_bolts::prelude::Rand;

const SPECIAL: [u8; 33] = [
    0, b'!', b'"', b'#', b'$', b'%', b'&', b'\'', b'(', b')', b'*', b'+', b',', b'-', b'.', b'/', b':', b';', b'<', b'=', b'>', b'?', b'@', b'\\', b'[', b']', b'^', b'_', b'`',
    b'{', b'|', b'}', 127,
];

pub fn mutate_special_insert<R: Rand>(rand: &mut R, stream: &mut TokenStream) -> bool {
    if stream.is_empty() {
        return false;
    }
    
    let start = rand.between(0, stream.len() - 1); 
    
    for token in &mut stream.tokens_mut()[start..] {
        if let TextToken::Text(data) = token {
            let c = rand.choose(SPECIAL).unwrap();
            let idx = rand.between(0, data.len());
            data.insert(idx, c);
            return true;
        }
    }
    
    false
}

pub fn mutate_special_replace<R: Rand>(rand: &mut R, stream: &mut TokenStream) -> bool {
    if stream.is_empty() {
        return false;
    }
    
    let start = rand.between(0, stream.len() - 1); 
    
    for token in &mut stream.tokens_mut()[start..] {
        if let TextToken::Text(data) = token {
            if data.is_empty() {
                continue;
            }
            
            let c = rand.choose(SPECIAL).unwrap();
            let idx = rand.between(0, data.len() - 1);
            data[idx] = c;
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
    fn test_special_insert() {
        let mut buffer = [0; 1024];
        let mut rand = StdRand::with_seed(current_nanos());
        let stream = "PORT 127,0,0,1,80,80\r\n".parse::<TokenStream>().unwrap();
        
        for _ in 0..10 {
            let mut stream = stream.clone();
            mutate_special_insert(&mut rand, &mut stream);
            let size = stream.serialize_content(&mut buffer);
            let s = std::str::from_utf8(&buffer[0..size]).unwrap();
            println!("{}", s);
        }
    }
    
    #[test]
    fn test_special_replace() {
        let mut buffer = [0; 1024];
        let mut rand = StdRand::with_seed(current_nanos());
        let stream = "PORT 127,0,0,1,80,80\r\n".parse::<TokenStream>().unwrap();
        
        for _ in 0..10 {
            let mut stream = stream.clone();
            mutate_special_replace(&mut rand, &mut stream);
            let size = stream.serialize_content(&mut buffer);
            let s = std::str::from_utf8(&buffer[0..size]).unwrap();
            println!("{}", s);
        }
    }
}
