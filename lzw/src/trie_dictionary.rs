// https://en.wikipedia.org/wiki/Trie
// https://docs.rs/trie-rs/latest/trie_rs/
// https://crates.io/crates/louds-rs

use crate::alphabets;
use crate::lzw_code;
use crate::lzw_token::LzwToken;
use crate::lzw_token::ValToken;
use crate::LzwSpec;
use std::cell::RefCell;
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
pub struct TrieNode<T: ValToken> {
    key: Option<LzwToken<T>>,
    value: Option<lzw_code::Code>,
    terminator: bool,
    children: HashMap<LzwToken<T>, RefCell<TrieNode<T>>>,
}

pub struct TrieDictionary<T: ValToken> {
    root: RefCell<TrieNode<T>>,
    // alphabet: Vec<char>,
    clear_code: bool,
    end_code: bool,
}

impl<T: ValToken> TrieNode<T> {
    pub fn new(key: LzwToken<T>, sequence_code: lzw_code::Code, terminator: bool) -> TrieNode<T> {
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

    pub fn add_child(&mut self, val: LzwToken<T>, sequence_code: lzw_code::Code, terminator: bool) {
        let inserted = self.children.insert(
            val,
            RefCell::new(TrieNode::new(val, sequence_code, terminator)),
        );
        match inserted {
            None => {}
            Some(old) => println!("Already had an entry!!! : {:?}", old),
        }
    }
}

impl<T: ValToken> TrieDictionary<T> {
    pub fn new(
        lzw_spec: LzwSpec,
        code_gen: &mut lzw_code::CodeGenerator,
        alphabet: Vec<LzwToken<T>>,
    ) -> TrieDictionary<T> {
        let mut new_trie = TrieDictionary {
            root: RefCell::new(TrieNode::new_root()),
            // alphabet,
            clear_code: lzw_spec.clear_code,
            end_code: lzw_spec.end_code,
        };

        let mut root = new_trie.root.borrow_mut();

        // ADD the alphabet
        for symbol in alphabet.iter() {
            if let Some(code) = code_gen.get_next_code() {
                root.add_child(*symbol, code, true);
                //println!("code: {}", code);
            } else {
                panic!("Base alphabet too large for starting bit width");
            }
        }

        // ADD the clear code control character
        if lzw_spec.clear_code {
            println!("Clearing");
            if let Some(code) = code_gen.get_next_code() {
                root.add_child(LzwToken::new_control_clear(), code, true);
                println!("code END: {}", code);
            } else {
                panic!("Base alphabet too large for starting bit width");
            }
        }

        // ADD the end code control character
        // TODO once the get_next_code is refactored, propogate error instead of manually panicking here
        if lzw_spec.end_code {
            println!("Ending");
            if let Some(code) = code_gen.get_next_code() {
                root.add_child(LzwToken::new_control_end(), code, true);
                println!("code END: {}", code);
            } else {
                panic!("Base alphabet too large for starting bit width");
            }
        }
        new_trie
    }

    pub fn run_encrypt<I>(&mut self, token_iter: I, code_gen: &mut lzw_code::CodeGenerator)
    where
        I: Iterator<Item = LzwToken<T>>,
    {
        let mut current_node = &self.root.borrow_mut();
        for token in token_iter {
            // If Token in current node children, jump down
            // If not, remember the current code
            //      add the child, using next code
            //      jump back to root
            if let Some(child) = current_node.children.get(&token) {
                let current_node = &child.borrow_mut();
            } else {
                let code = current_node.value.unwrap();
                current_node.add_child(token, code_gen.get_next_code().unwrap(), true);
            }
        }
    }
}

// impl<T: HashableToken> TrieDictionary<T> {
//     // really we want a reference to an iterator
//     // or make the struct more stateful and expose a step(char) or similar
//     pub fn fetch_code_and_insert(
//         &mut self,
//         search_seq: &[Token<T>],
//         next_code: lzw_code::Code, // TODO: Take a reference to the code generator so that codes advance only when needed
//     ) -> LookupResult {
//         let mut current_node = &mut self.root;
//         let mut fetched_code: Option<lzw_code::Code> = Option::None;
//         let mut consumed_tokens: u32 = 0;
//         for symbol in search_seq.iter() {
//             if current_node.children.contains_key(symbol) {
//                 current_node = current_node.children.get_mut(symbol).unwrap();
//                 consumed_tokens += 1;
//             } else {
//                 fetched_code = current_node.value;
//                 current_node.add_child(*symbol, next_code, true);
//                 current_node.terminator = false;
//             }
//         }
//         match fetched_code {
//             Some(code) => LookupResult::new(code, consumed_tokens),
//             None => panic!("didn't fetch a code"),
//         }
//     }

//     pub fn _search(&self, search_seq: &[Token<T>]) -> Option<lzw_code::Code> {
//         let mut current_node = &self.root;
//         for symbol in search_seq.iter() {
//             if current_node.children.contains_key(symbol) {
//                 current_node = current_node.children.get(symbol).unwrap();
//             } else {
//                 return None;
//             }
//         }
//         current_node.value
//     }

//     pub fn _insert(&mut self, input_seq: &[Token<T>], sequence_code: lzw_code::Code) {
//         let mut current_node = &mut self.root;
//         for symbol in input_seq.iter() {
//             if current_node.children.contains_key(symbol) {
//                 current_node = current_node.children.get_mut(symbol).unwrap();
//             } else {
//                 // Add child, move down into it
//                 current_node.add_child(*symbol, sequence_code, true);
//                 current_node.terminator = false;
//                 current_node = current_node.children.get_mut(symbol).unwrap();
//             }
//         }
//     }

//     pub fn new(
//         lzw_spec: LzwSpec,
//         code_gen: &mut lzw_code::CodeGenerator,
//         alphabet: Vec<Token<T>>,
//     ) -> TrieDictionary<T> {
//         let mut new_trie = TrieDictionary {
//             root: TrieNode::new_root(),
//             // alphabet,
//             clear_code: lzw_spec.clear_code,
//             end_code: lzw_spec.end_code,
//         };

//         println!("Size of initial dictionary before alphabet: {}", {
//             new_trie.root.children.len()
//         });

//         // ADD the alphabet
//         for symbol in alphabet.iter() {
//             if let Some(code) = code_gen.get_next_code() {
//                 new_trie.root.add_child(*symbol, code, true);
//                 //println!("code: {}", code);
//             } else {
//                 panic!("Base alphabet too large for starting bit width");
//             }
//         }

//         println!("Size of initial dictionary before control: {}", {
//             new_trie.root.children.len()
//         });

//         // ADD the clear code control character
//         if lzw_spec.clear_code {
//             if let Some(code) = code_gen.get_next_code() {
//                 new_trie.root.add_child(
//                     Token::new_control(crate::lzw_token::ControlToken::Clear),
//                     code,
//                     true,
//                 );
//                 println!("code END: {}", code);
//             } else {
//                 panic!("Base alphabet too large for starting bit width");
//             }
//         }

//         // ADD the end code control character
//         if lzw_spec.end_code {
//             if let Some(code) = code_gen.get_next_code() {
//                 new_trie.root.add_child(
//                     Token::new_control(crate::lzw_token::ControlToken::End),
//                     code,
//                     true,
//                 );
//                 println!("code END: {}", code);
//             } else {
//                 panic!("Base alphabet too large for starting bit width");
//             }
//         }

//         println!("Size of initial dictionary: {}", {
//             new_trie.root.children.len()
//         });
//         new_trie
//     }
// }

// #[cfg(test)]
// mod tests {

//     use super::*;
//     use crate::{lzw_code::CodeGenerator, lzw_token};

//     const _TEST_SPEC: LzwSpec = LzwSpec {
//         alphabet: alphabets::Alphabet::_Test,
//         variable_width: false,
//         width: 12,
//         min_width: 12,
//         max_width: 12,
//         end_code: true,
//         clear_code: true,
//         pack_msb_first: true,
//         early_change: false,
//     };

//     const ASCII_SPEC: LzwSpec = LzwSpec {
//         alphabet: alphabets::Alphabet::Ascii,
//         variable_width: false,
//         width: 12,
//         min_width: 12,
//         max_width: 12,
//         end_code: true,
//         clear_code: true,
//         pack_msb_first: true,
//         early_change: false,
//     };

//     #[test]
//     fn search_initial_dict() {
//         let alphabet = alphabets::generate_ascii();
//         let alpha_len = alphabet.len();
//         let mut code_gen = CodeGenerator::new(ASCII_SPEC);
//         let dict = TrieDictionary::new(ASCII_SPEC, &mut code_gen, alphabet);

//         let mut ver_code_gen = CodeGenerator::new(ASCII_SPEC);
//         let mut other_alphabet = alphabets::generate_ascii();
//         other_alphabet.reverse();
//         for _ in 0..alpha_len {
//             let a_token = other_alphabet.pop().unwrap();
//             match dict._search(&[a_token]) {
//                 None => panic!("Expected entry not in dict"),
//                 Some(code_result) => assert_eq!(ver_code_gen.get_next_code().unwrap(), code_result),
//             }
//         }
//         match dict._search(&[lzw_token::Token::new_control(
//             lzw_token::ControlToken::Clear,
//         )]) {
//             None => panic!("Expected Clear CODE not in dict"),
//             Some(code_result) => assert_eq!(ver_code_gen.get_next_code().unwrap(), code_result),
//         }
//         match dict._search(&[lzw_token::Token::new_control(lzw_token::ControlToken::End)]) {
//             None => panic!("Expected End CODE not in dict"),
//             Some(code_result) => assert_eq!(ver_code_gen.get_next_code().unwrap(), code_result),
//         }
//     }

//     #[test]
//     #[should_panic]
//     fn search_not_present() {
//         let alphabet = alphabets::generate_ascii();
//         let mut code_gen = CodeGenerator::new(ASCII_SPEC);
//         let dict = TrieDictionary::new(ASCII_SPEC, &mut code_gen, alphabet);

//         let tok_seq = &[Token::new('A'), Token::new('B')];
//         let expected_code = code_gen.get_next_code().unwrap();
//         match dict._search(tok_seq) {
//             None => panic!("Expected End CODE not in dict"),
//             Some(code_result) => assert_eq!(expected_code, code_result),
//         }
//     }

//     #[test]
//     fn insert_test() {
//         let alphabet = alphabets::generate_ascii();
//         let mut code_gen = CodeGenerator::new(ASCII_SPEC);
//         let mut dict = TrieDictionary::new(ASCII_SPEC, &mut code_gen, alphabet);

//         let tok_seq = &[Token::new('A'), Token::new('B')];
//         let expected_code = code_gen.get_next_code().unwrap();
//         dict._insert(tok_seq, expected_code);
//         match dict._search(tok_seq) {
//             None => panic!("Expected End CODE not in dict"),
//             Some(code_result) => assert_eq!(expected_code, code_result),
//         }
//     }

//     #[test]
//     fn insert_already_present() {
//         let alphabet = alphabets::generate_ascii();
//         let mut code_gen = CodeGenerator::new(ASCII_SPEC);
//         let mut dict = TrieDictionary::new(ASCII_SPEC, &mut code_gen, alphabet);

//         let tok_seq = &[Token::new('A'), Token::new('B')];
//         let expected_code = code_gen.get_next_code().unwrap();
//         dict._insert(tok_seq, expected_code);

//         let unexpected_code = code_gen.get_next_code().unwrap();
//         dict._insert(tok_seq, unexpected_code);

//         match dict._search(tok_seq) {
//             None => panic!("Expected End CODE not in dict"),
//             Some(code_result) => assert_eq!(expected_code, code_result),
//         }
//     }

//     #[test]
//     fn fetch_and_insert() {
//         let alphabet = alphabets::generate_ascii();
//         let mut code_gen = CodeGenerator::new(ASCII_SPEC);
//         let mut dict = TrieDictionary::new(ASCII_SPEC, &mut code_gen, alphabet);

//         let existing_sub_seq = &[Token::new('A')];
//         let expected_fetched_code = dict._search(existing_sub_seq).unwrap();

//         let fetch_insert_seq = &[Token::new('A'), Token::new('B')];
//         let expected_insert_code = code_gen.get_next_code().unwrap();
//         let fetched_result = dict.fetch_code_and_insert(fetch_insert_seq, expected_insert_code);

//         assert_eq!(expected_fetched_code, fetched_result.code);
//         assert_eq!(
//             fetch_insert_seq.len() as u32 - 1, // n-1 tokens should be consumed
//             fetched_result.consumed_tokens
//         );

//         let inserted_code = dict._search(fetch_insert_seq).unwrap();
//         assert_eq!(expected_insert_code, inserted_code);
//     }
// }
