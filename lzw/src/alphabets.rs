use crate::lzw_token::{ControlToken, Token};
use crate::ArgAlphabet;

#[derive(Debug, Copy, Clone)]
pub enum Alphabet {
    _Test,
    Ascii,
    // TODO add more
}

impl Alphabet {
    pub fn new(alpha: ArgAlphabet) -> Alphabet {
        match alpha {
            ArgAlphabet::_Test => Alphabet::_Test,
            ArgAlphabet::Ascii => Alphabet::Ascii,
        }
    }
}

//TODO in future refactor to Vec<Token> so that non text files e.g. images can be compressed.
// pub fn produce_alphabet(alpha: Alphabet) -> Vec<Token> {
//     match alpha {
//         Alphabet::_Test => generate_test_alphabet(),
//         Alphabet::Ascii => generate_ascii(),
//     }
// }

pub fn generate_ascii() -> Vec<Token<char>> {
    let printable_chars: String = String::from(" !\"#$%&\'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~");
    let alphabet = printable_chars.chars();
    let mut res: Vec<Token<char>> = alphabet.map(|c| Token::new(c)).collect();
    res.push(Token::new_control(ControlToken::CLEAR));
    res.push(Token::new_control(ControlToken::END));
    println!("Length of initial alphabet {}", res.len());
    res
}

fn generate_test_alphabet<T>() -> Vec<T> {
    let alphabet: Vec<T> = Vec::new();
    alphabet
}
