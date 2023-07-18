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
pub fn produce_alphabet(alpha: Alphabet) -> Vec<char> {
    match alpha {
        Alphabet::_Test => generate_test_alphabet(),
        Alphabet::Ascii => generate_ascii(),
    }
}

fn generate_ascii() -> Vec<char> {
    let printable_chars: String = String::from(" !\"#$%&\'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~");
    let alphabet: Vec<char> = printable_chars.chars().collect();
    println!("Length of initial alphabet {}", alphabet.len());
    alphabet
}

fn generate_test_alphabet() -> Vec<char> {
    let alphabet: Vec<char> = Vec::new();
    alphabet
}
