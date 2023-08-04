use crate::lzw_token::AsciiToken;
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

// TODO: fix this if it is needed?
// pub fn produce_alphabet(alpha: Alphabet) -> Box<dynVec<Token<T>> {
//     match alpha {
//         Alphabet::_Test => generate_test_alphabet(),
//         Alphabet::Ascii => generate_ascii(),
//     }
// }

pub fn generate_ascii() -> Vec<AsciiToken> {
    tracing::debug!("Creating Ascii Printable Alphabet");
    let printable_chars: String = String::from(" !\"#$%&\'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~");
    let alphabet = printable_chars.chars();
    let res: Vec<AsciiToken> = alphabet.map(AsciiToken::new).collect();
    res
}

// fn generate_test_alphabet<T>() -> Vec<T> {
//     let alphabet: Vec<T> = Vec::new();
//     alphabet
// }
