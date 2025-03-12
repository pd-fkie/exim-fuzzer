mod split;
mod crossover;
mod delete;
mod common;
mod copy;
mod swap;
mod repeat;
mod random;
mod interesting;
mod special;
mod dict;
mod flip;
mod truncate;

pub use split::*;
pub use crossover::*;
pub use delete::*;
pub use copy::*;
pub use swap::*;
pub use repeat::*;
pub use random::*;
pub use interesting::*;
pub use special::*;
pub use dict::*;
pub use flip::*;
pub use truncate::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::Packet;
    use crate::tokens::TokenStream;
    use libafl::prelude::Tokens;
    use libafl_bolts::prelude::{StdRand, current_nanos, Rand};
    
    #[test]
    fn fuzz_mutators() {
        let stream = "200 fuck my shit up\r\nPORT 127,0,0,1,80,80\r\n12 + 12 = 24".parse::<TokenStream>().unwrap();
        let mut dict = Tokens::new();
        dict.add_tokens([
            &b"X".to_vec(),
            &b"Y".to_vec(),
            &b"Z".to_vec(),
        ]);
        let mut rand = StdRand::with_seed(current_nanos());
        let mut count = 0;
        const MAX_LEN: usize = 128;
        
        loop {
            count += 1;
            println!("iter {}", count);
            
            let mut stream = stream.clone();
            
            for _ in 0..1000 {
                let mutation = rand.zero_upto(18);
                
                let mutated = match mutation {
                    0 => mutate_copy(&mut rand, &mut stream, MAX_LEN),
                    1 => {
                        let other = stream.clone();
                        mutate_crossover_insert(&mut rand, &mut stream, &other, MAX_LEN)
                    },
                    2 => {
                        let other = stream.clone();
                        mutate_crossover_replace(&mut rand, &mut stream, &other, MAX_LEN)
                    },
                    3 => mutate_delete(&mut rand, &mut stream),
                    4 => mutate_dict_insert(&mut rand, &mut stream, &dict, MAX_LEN),
                    5 => mutate_dict_replace(&mut rand, &mut stream, &dict),
                    6 => mutate_flip(&mut rand, &mut stream),
                    7 => mutate_interesting(&mut rand, &mut stream),
                    8 => mutate_random_insert(&mut rand, &mut stream, MAX_LEN),
                    9 => mutate_random_replace(&mut rand, &mut stream),
                    10 => mutate_repeat_char::<_, 8>(&mut rand, &mut stream),
                    11 => mutate_repeat_token::<_, 8>(&mut rand, &mut stream, MAX_LEN),
                    12 => mutate_special_insert(&mut rand, &mut stream),
                    13 => mutate_special_replace(&mut rand, &mut stream),
                    14 => mutate_split(&mut rand, &mut stream, MAX_LEN),
                    15 => mutate_swap_constants(&mut rand, &mut stream, &dict),
                    16 => mutate_swap_tokens(&mut rand, &mut stream),
                    17 => mutate_swap_words(&mut rand, &mut stream),
                    18 => mutate_truncate(&mut rand, &mut stream),
                    _ => unreachable!(),
                };
                
                if mutated {
                    if stream.len() > MAX_LEN {
                        panic!("Mutation #{} went out of bounds for number of tokens", mutation);
                    }
                    
                    for token in stream.tokens() {
                        if !token.verify() {
                            panic!("Mutation #{} produced invalid token: {:?} (full stream: {:?})", mutation, token, stream);
                        }
                    }
                }
            }
        }
    }
    
    #[test]
    fn test_generation() {
        let mut stream = "".parse::<TokenStream>().unwrap();
        let mut dict = Tokens::new();
        dict.add_tokens([
            &b"<TOKEN>".to_vec(),
        ]);
        let mut rand = StdRand::with_seed(current_nanos());
        const MAX_LEN: usize = 128;
        
        for _ in 0..10 {
            match rand.zero_upto(18) {
                0 => mutate_copy(&mut rand, &mut stream, MAX_LEN),
                1 => {
                    let other = stream.clone();
                    mutate_crossover_insert(&mut rand, &mut stream, &other, MAX_LEN)
                },
                2 => {
                    let other = stream.clone();
                    mutate_crossover_replace(&mut rand, &mut stream, &other, MAX_LEN)
                },
                3 => mutate_delete(&mut rand, &mut stream),
                4 => mutate_dict_insert(&mut rand, &mut stream, &dict, MAX_LEN),
                5 => mutate_dict_replace(&mut rand, &mut stream, &dict),
                6 => mutate_flip(&mut rand, &mut stream),
                7 => mutate_interesting(&mut rand, &mut stream),
                8 => mutate_random_insert(&mut rand, &mut stream, MAX_LEN),
                9 => mutate_random_replace(&mut rand, &mut stream),
                10 => mutate_repeat_char::<_, 8>(&mut rand, &mut stream),
                11 => mutate_repeat_token::<_, 8>(&mut rand, &mut stream, MAX_LEN),
                12 => mutate_special_insert(&mut rand, &mut stream),
                13 => mutate_special_replace(&mut rand, &mut stream),
                14 => mutate_split(&mut rand, &mut stream, MAX_LEN),
                15 => mutate_swap_constants(&mut rand, &mut stream, &dict),
                16 => mutate_swap_tokens(&mut rand, &mut stream),
                17 => mutate_swap_words(&mut rand, &mut stream),
                18 => mutate_truncate(&mut rand, &mut stream),
                _ => unreachable!(),
            };
        }
        
        let mut total_len = 0;
        for token in stream.tokens() {
            total_len += token.len();
        }
        
        let mut buffer = vec![0; total_len];
        stream.serialize_content(&mut buffer);
        
        println!("{}", std::str::from_utf8(&buffer).unwrap());
    }
    
    #[test]
    fn test_mutation() {
        let mut stream = "PORT 127,0,0,1,80,80\r\n".parse::<TokenStream>().unwrap();
        let mut dict = Tokens::new();
        dict.add_tokens([
            &b"<TOKEN>".to_vec(),
        ]);
        let mut rand = StdRand::with_seed(current_nanos());
        const MAX_LEN: usize = 128;
        
        for _ in 0..2 {
            match rand.zero_upto(18) {
                0 => mutate_copy(&mut rand, &mut stream, MAX_LEN),
                1 => {
                    let other = stream.clone();
                    mutate_crossover_insert(&mut rand, &mut stream, &other, MAX_LEN)
                },
                2 => {
                    let other = stream.clone();
                    mutate_crossover_replace(&mut rand, &mut stream, &other, MAX_LEN)
                },
                3 => mutate_delete(&mut rand, &mut stream),
                4 => mutate_dict_insert(&mut rand, &mut stream, &dict, MAX_LEN),
                5 => mutate_dict_replace(&mut rand, &mut stream, &dict),
                6 => mutate_flip(&mut rand, &mut stream),
                7 => mutate_interesting(&mut rand, &mut stream),
                8 => mutate_random_insert(&mut rand, &mut stream, MAX_LEN),
                9 => mutate_random_replace(&mut rand, &mut stream),
                10 => mutate_repeat_char::<_, 8>(&mut rand, &mut stream),
                11 => mutate_repeat_token::<_, 8>(&mut rand, &mut stream, MAX_LEN),
                12 => mutate_special_insert(&mut rand, &mut stream),
                13 => mutate_special_replace(&mut rand, &mut stream),
                14 => mutate_split(&mut rand, &mut stream, MAX_LEN),
                15 => mutate_swap_constants(&mut rand, &mut stream, &dict),
                16 => mutate_swap_tokens(&mut rand, &mut stream),
                17 => mutate_swap_words(&mut rand, &mut stream),
                18 => mutate_truncate(&mut rand, &mut stream),
                _ => unreachable!(),
            };
        }
        
        let mut total_len = 0;
        for token in stream.tokens() {
            total_len += token.len();
        }
        
        let mut buffer = vec![0; total_len];
        stream.serialize_content(&mut buffer);
        
        println!("{:?}", std::str::from_utf8(&buffer).unwrap());
    }
}
