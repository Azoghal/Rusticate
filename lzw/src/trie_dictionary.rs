// https://en.wikipedia.org/wiki/Trie
// https://docs.rs/trie-rs/latest/trie_rs/
// https://crates.io/crates/louds-rs

use crate::alphabets;
use crate::lzw_codes;
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
pub struct TrieNode {
    key: Option<char>,
    value: Option<lzw_codes::Code>,
    terminator: bool,
    children: HashMap<char, TrieNode>,
}

pub struct TrieDictionary {
    root: TrieNode,
    // alphabet: Vec<char>,
    clear_code: bool,
    end_code: bool,
}

impl TrieNode {
    pub fn new(key: char, sequence_code: lzw_codes::Code, terminator: bool) -> TrieNode {
        TrieNode {
            key: Option::Some(key),
            value: Some(sequence_code),
            terminator,
            children: HashMap::new(),
        }
    }

    pub fn new_root() -> TrieNode {
        TrieNode {
            key: Option::None,
            value: Option::None,
            terminator: true,
            children: HashMap::new(),
        }
    }

    pub fn add_child(&mut self, val: &char, sequence_code: lzw_codes::Code, terminator: bool) {
        self.children
            .insert(*val, TrieNode::new(*val, sequence_code, terminator));
    }
}

impl TrieDictionary {
    // really we want a reference to an iterator
    // or make the struct more stateful and expose a step(char) or similar
    pub fn fetch_code_and_insert(
        &mut self,
        search_seq: &[char],
        next_code: lzw_codes::Code,
    ) -> lzw_codes::Code {
        let mut current_node = &mut self.root;
        let mut fetched_code: Option<lzw_codes::Code> = Option::None;
        for symbol in search_seq.iter() {
            if current_node.children.contains_key(symbol) {
                current_node = current_node.children.get_mut(symbol).unwrap();
            } else {
                fetched_code = current_node.value;
                current_node.add_child(symbol, next_code, true);
                current_node.terminator = false;
            }
        }
        match fetched_code {
            Some(code) => code,
            None => panic!("didn't fetch a code"),
        }
    }

    pub fn search(self, search_seq: &[char]) -> Option<lzw_codes::Code> {
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

    pub fn insert(&mut self, input_seq: &[char], sequence_code: lzw_codes::Code) {
        let mut current_node = &mut self.root;
        for symbol in input_seq.iter() {
            if current_node.children.contains_key(symbol) {
                current_node = current_node.children.get_mut(symbol).unwrap();
            } else {
                // Add child, move down into it
                current_node.add_child(symbol, sequence_code, true);
                current_node.terminator = false;
                current_node = current_node.children.get_mut(symbol).unwrap();
            }
        }
    }

    pub fn new(lzw_spec: LzwSpec, code_gen: &mut lzw_codes::CodeGenerator) -> TrieDictionary {
        let mut new_trie = TrieDictionary {
            root: TrieNode::new_root(),
            // alphabet,
            clear_code: lzw_spec.clear_code,
            end_code: lzw_spec.end_code,
        };

        // TODO make alphabets vec of something more than char so non-character escape codes etc
        let alphabet: Vec<char> = alphabets::produce_alphabet(lzw_spec.alphabet);

        for symbol in alphabet.iter() {
            let code_result: Option<lzw_codes::Code> = code_gen.get_next_code();
            match code_result {
                None => panic!("Base alphabet too large for starting bit width"),
                Some(code) => {
                    new_trie.root.add_child(symbol, code, true);
                    println!("code: {}", code);
                }
            }
        }
        new_trie
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lzw_codes::CodeGenerator;

    const TEST_SPEC: LzwSpec = LzwSpec {
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
    fn insert_seqs() {
        let mut code_gen = CodeGenerator::new(TEST_SPEC);
        let mut dict = TrieDictionary::new(TEST_SPEC, &mut code_gen);
        let code = code_gen.get_next_code().unwrap();
        dict.insert(&['a'], code);
        match dict.search(&['a']) {
            None => panic!("Expected entry not in dict"),
            Some(code_result) => assert_eq!(code, code_result),
        }
    }

    #[test]
    fn fetch_existing_code() {
        let mut code_gen = CodeGenerator::new(ASCII_SPEC);
        let mut dict = TrieDictionary::new(ASCII_SPEC, &mut code_gen);

        let Some(code) = code_gen.get_next_code() else{
            panic!("Out of codes");
        };
        println!("Code for first insert and fetch {}", code);

        let fetched_code = dict.fetch_code_and_insert(&['a', 'b'], code);
        assert_eq!(fetched_code.get_code(), 65);

        let Some(code) = code_gen.get_next_code() else{
            panic!("Out of codes");
        };
        println!("Code for second insert and fetch {}", code);
        let inserted_and_fetched_code = dict.fetch_code_and_insert(&['a', 'b', 'c'], code);
        assert_eq!(inserted_and_fetched_code.get_code(), 95);
    }
}
