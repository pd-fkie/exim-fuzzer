use crate::tokens::{TokenStream, TextToken, mutators::common::copy_vec};
use libafl_bolts::prelude::Rand;

const INTERESTING: [&[u8]; 33] = [
    b"0",
    b"-1",
    // 0x7F
    b"127",
    // 0x80
    b"-128",
    b"128",
    // 0xFF
    b"255",
    // 0x7FFF
    b"32767",
    // 0x8000
    b"-32768",
    b"32768",
    // 0xFFFF
    b"65535",
    // 0x7FFFFFFF
    b"2147483647",
    // 0x80000000
    b"2147483648",
    b"-2147483648",
    // 0xFFFFFFFF
    b"4294967295",
    // 0x7FFFFFFFFFFFFFFF
    b"9223372036854775807",
    // 0x8000000000000000
    b"9223372036854775808",
    b"-9223372036854775808",
    // 0xFFFFFFFFFFFFFFFF
    b"18446744073709551615",
    // powers of 2
    b"16",
    b"32",
    b"64",
    b"256",
    b"512",
    b"1024",
    b"4096",
    b"65536",
    // random bullshit go
    b"1",
    b"100",
    b"-129",
    b"1000",
    b"-100663046",
    b"-32769",
    b"100663045",
];

pub fn mutate_interesting<R: Rand>(rand: &mut R, stream: &mut TokenStream) -> bool {
    if stream.is_empty() {
        return false;
    }
    
    let start = rand.between(0, stream.len() - 1);
    
    for token in &mut stream.tokens_mut()[start..] {
        if let TextToken::Number(data) = token {
            copy_vec(data, rand.choose(INTERESTING).unwrap());
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
    fn test_interesting() {
        let mut buffer = [0; 1024];
        let mut rand = StdRand::with_seed(current_nanos());
        let stream = "PORT 127,0,0,1,80,80\r\n".parse::<TokenStream>().unwrap();
        
        for _ in 0..10 {
            let mut stream = stream.clone();
            mutate_interesting(&mut rand, &mut stream);
            let size = stream.serialize_content(&mut buffer);
            let s = std::str::from_utf8(&buffer[0..size]).unwrap();
            println!("{}", s);
        }
    }
}
