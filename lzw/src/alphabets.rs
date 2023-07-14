use crate::Alphabet;

//TODO in future refactor to Vec<Token> so that non text files e.g. images can be compressed.
pub fn produce_alphabet(alpha: Alphabet) -> Vec<char> {
    match alpha {
        Alphabet::Ascii => generate_ascii(),
    }
}

fn generate_ascii() -> Vec<char> {
    let printable_chars: String = String::from(" !\"#$%&\'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~");
    let alphabet: Vec<char> = printable_chars.chars().collect();
    alphabet
}
