// https://en.wikipedia.org/wiki/Trie
// https://docs.rs/trie-rs/latest/trie_rs/
// https://crates.io/crates/louds-rs

use crate::alphabets;
use crate::lzw_code;
use crate::lzw_token::{HashableToken, Token};
use crate::LzwSpec;
use std::collections::HashMap;

// TODO implement with generics
// TODO compare speed against trie_rs

/* WHY USE A TRIE FOR LZW?

1. "Buffer input characters in a sequence w until w+next_character is not in the dictionary"
    We need a quick way to check (character by character) whether a sequence is in the dictionary
    In a trie, this means consuming characters from the input and traversing the trie until a terminator is found
    This is O(|w|) to find w+next_character that is not in the dictionary
2. "Add w+next_character to the dictionary"
    We will have already consumed w and traversed the trie, so inserting the new code is O(1) for our case
3. "Start buffering again with the next character"
    Now we forget about the w we found, and return to step 1

The trie is ideal as the *sequential* lookup/search takes linear time, and still lets us insert in O(1)
*/

/* FURTHER CONSIDERATIONS
    We don't need a delete function as either dictionary entries are never removed,
    or all of the non-starting-dictionary elements are removed when the max code size is reached.
    In this case, we can just reinitialize the dictionary
*/

#[derive(Debug)]
pub struct TrieNode<T: HashableToken> {
    key: Option<Token<T>>,
    value: Option<lzw_code::Code>,
    terminator: bool,
    children: HashMap<Token<T>, TrieNode<T>>,
}

pub struct TrieDictionary<T: HashableToken> {
    root: TrieNode<T>,
    // alphabet: Vec<char>,
    clear_code: bool,
    end_code: bool,
}

impl<T: HashableToken> TrieNode<T> {
    pub fn new(key: Token<T>, sequence_code: lzw_code::Code, terminator: bool) -> TrieNode<T> {
        TrieNode {
            key: Option::Some(key),
            value: Some(sequence_code),
            terminator,
            children: HashMap::new(),
        }
    }

    pub fn new_root() -> TrieNode<T> {
        TrieNode {
            key: Option::None,
            value: Option::None,
            terminator: true,
            children: HashMap::new(),
        }
    }

    pub fn add_child(&mut self, val: Token<T>, sequence_code: lzw_code::Code, terminator: bool) {
        let inserted = self
            .children
            .insert(val, TrieNode::new(val, sequence_code, terminator));
        match inserted {
            None => {}
            Some(old) => println!("Already had an entry!!! : {:?}", old),
        }
    }
}

impl<T: HashableToken> TrieDictionary<T> {
    // really we want a reference to an iterator
    // or make the struct more stateful and expose a step(char) or similar
    pub fn fetch_code_and_insert(
        &mut self,
        search_seq: &[Token<T>],
        next_code: lzw_code::Code,
    ) -> lzw_code::Code {
        let mut current_node = &mut self.root;
        let mut fetched_code: Option<lzw_code::Code> = Option::None;
        for symbol in search_seq.iter() {
            if current_node.children.contains_key(symbol) {
                current_node = current_node.children.get_mut(symbol).unwrap();
            } else {
                fetched_code = current_node.value;
                current_node.add_child(*symbol, next_code, true);
                current_node.terminator = false;
            }
        }
        match fetched_code {
            Some(code) => code,
            None => panic!("didn't fetch a code"),
        }
    }

    pub fn _search(&self, search_seq: &[Token<T>]) -> Option<lzw_code::Code> {
        let mut current_node = &self.root;
        for symbol in search_seq.iter() {
            if current_node.children.contains_key(symbol) {
                current_node = current_node.children.get(symbol).unwrap();
            } else {
                return None;
            }
        }
        current_node.value
    }

    pub fn _insert(&mut self, input_seq: &[Token<T>], sequence_code: lzw_code::Code) {
        let mut current_node = &mut self.root;
        for symbol in input_seq.iter() {
            if current_node.children.contains_key(symbol) {
                current_node = current_node.children.get_mut(symbol).unwrap();
            } else {
                // Add child, move down into it
                current_node.add_child(*symbol, sequence_code, true);
                current_node.terminator = false;
                current_node = current_node.children.get_mut(symbol).unwrap();
            }
        }
    }

    pub fn new(
        lzw_spec: LzwSpec,
        code_gen: &mut lzw_code::CodeGenerator,
        alphabet: Vec<Token<T>>,
    ) -> TrieDictionary<T> {
        let mut new_trie = TrieDictionary {
            root: TrieNode::new_root(),
            // alphabet,
            clear_code: lzw_spec.clear_code,
            end_code: lzw_spec.end_code,
        };

        println!("Size of initial dictionary before alphabet: {}", {
            new_trie.root.children.len()
        });

        // ADD the alphabet
        for symbol in alphabet.iter() {
            if let Some(code) = code_gen.get_next_code() {
                new_trie.root.add_child(*symbol, code, true);
                //println!("code: {}", code);
            } else {
                panic!("Base alphabet too large for starting bit width");
            }
        }

        println!("Size of initial dictionary before control: {}", {
            new_trie.root.children.len()
        });

        // ADD the clear code control character
        if lzw_spec.clear_code {
            if let Some(code) = code_gen.get_next_code() {
                new_trie.root.add_child(
                    Token::new_control(crate::lzw_token::ControlToken::Clear),
                    code,
                    true,
                );
                println!("code END: {}", code);
            } else {
                panic!("Base alphabet too large for starting bit width");
            }
        }

        // ADD the end code control character
        if lzw_spec.end_code {
            if let Some(code) = code_gen.get_next_code() {
                new_trie.root.add_child(
                    Token::new_control(crate::lzw_token::ControlToken::End),
                    code,
                    true,
                );
                println!("code END: {}", code);
            } else {
                panic!("Base alphabet too large for starting bit width");
            }
        }

        println!("Size of initial dictionary: {}", {
            new_trie.root.children.len()
        });
        new_trie
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{
        lzw_code::{Code, CodeGenerator},
        lzw_token,
    };

    const _TEST_SPEC: LzwSpec = LzwSpec {
        alphabet: alphabets::Alphabet::_Test,
        variable_width: false,
        width: 12,
        min_width: 12,
        max_width: 12,
        end_code: true,
        clear_code: true,
        pack_msb_first: true,
        early_change: false,
    };

    const ASCII_SPEC: LzwSpec = LzwSpec {
        alphabet: alphabets::Alphabet::Ascii,
        variable_width: false,
        width: 12,
        min_width: 12,
        max_width: 12,
        end_code: true,
        clear_code: true,
        pack_msb_first: true,
        early_change: false,
    };

    #[test]
    fn control_hashed() {
        let ascii_char = Token::<char>::new('a');
        let clear = Token::<char>::new_control(lzw_token::ControlToken::Clear);
        let end = Token::<char>::new_control(lzw_token::ControlToken::End);

        let mut my_map: HashMap<Token<char>, u8> = HashMap::new();

        my_map.insert(ascii_char, 17);
        my_map.insert(clear, 27);
        my_map.insert(end, 39);

        println!("Number of key value pairs: {}", my_map.len());
        assert_ne!(my_map.get(&ascii_char), my_map.get(&clear))
    }

    #[test]
    fn search_initial_dict() {
        let alphabet = alphabets::generate_ascii();
        let alpha_len = alphabet.len();
        let mut code_gen = CodeGenerator::new(ASCII_SPEC);
        let dict = TrieDictionary::new(ASCII_SPEC, &mut code_gen, alphabet);

        let mut ver_code_gen = CodeGenerator::new(ASCII_SPEC);
        let mut other_alphabet = alphabets::generate_ascii();
        other_alphabet.reverse();
        for _ in 0..alpha_len {
            let a_token = other_alphabet.pop().unwrap();
            match dict._search(&[a_token]) {
                None => panic!("Expected entry not in dict"),
                Some(code_result) => assert_eq!(ver_code_gen.get_next_code().unwrap(), code_result),
            }
        }
        match dict._search(&[lzw_token::Token::new_control(
            lzw_token::ControlToken::Clear,
        )]) {
            None => panic!("Expected Clear CODE not in dict"),
            Some(code_result) => assert_eq!(ver_code_gen.get_next_code().unwrap(), code_result),
        }
        match dict._search(&[lzw_token::Token::new_control(lzw_token::ControlToken::End)]) {
            None => panic!("Expected End CODE not in dict"),
            Some(code_result) => assert_eq!(ver_code_gen.get_next_code().unwrap(), code_result),
        }
    }

    #[test]
    fn insert_test() {
        let alphabet = alphabets::generate_ascii();
        let mut code_gen = CodeGenerator::new(ASCII_SPEC);
        let mut dict = TrieDictionary::new(ASCII_SPEC, &mut code_gen, alphabet);

        let tok_seq = &[Token::new('A'), Token::new('B')];
        let expected_code = code_gen.get_next_code().unwrap();
        dict._insert(tok_seq, expected_code);
        match dict._search(tok_seq) {
            None => panic!("Expected End CODE not in dict"),
            Some(code_result) => assert_eq!(expected_code, code_result),
        }
    }

    // #[test]
    // fn fetch_existing_code() {
    //     let mut code_gen = CodeGenerator::new(ASCII_SPEC);
    //     let mut dict = TrieDictionary::new(ASCII_SPEC, &mut code_gen);

    //     let Some(code) = code_gen.get_next_code() else{
    //         panic!("Out of codes");
    //     };
    //     println!("Code for first insert and fetch {}", code);

    //     let fetched_code = dict.fetch_code_and_insert(&['a', 'b'], code);
    //     assert_eq!(fetched_code.get_code(), 65);

    //     let Some(code) = code_gen.get_next_code() else{
    //         panic!("Out of codes");
    //     };
    //     println!("Code for second insert and fetch {}", code);
    //     let inserted_and_fetched_code = dict.fetch_code_and_insert(&['a', 'b', 'c'], code);
    //     assert_eq!(inserted_and_fetched_code.get_code(), 95);
    // }
}
