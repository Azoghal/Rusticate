use mutable_trie::{Token, TrieKey};

pub enum AlphabetError {
    Generate(String),
}

pub trait Alphabetable<T: TrieKey> {
    fn generate() -> Result<Vec<Token<T>>, AlphabetError> {
        Err(AlphabetError::Generate(String::from(
            "Generate not implemented for type",
        )))
    }
}

impl Alphabetable<char> for char {
    fn generate() -> Result<Vec<Token<char>>, AlphabetError> {
        let printable_chars: String = String::from(" !\"#$%&\'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~");
        let alphabet = printable_chars.chars();
        let res: Vec<Token<char>> = alphabet.map(Token::Value).collect();
        Ok(res)
    }
}

impl Alphabetable<i32> for i32 {
    fn generate() -> Result<Vec<Token<i32>>, AlphabetError> {
        let res: Vec<Token<i32>> = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
            .into_iter()
            .map(Token::Value)
            .collect();
        Ok(res)
    }
}
