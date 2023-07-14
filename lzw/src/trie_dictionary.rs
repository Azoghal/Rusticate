// https://en.wikipedia.org/wiki/Trie
// https://docs.rs/trie-rs/latest/trie_rs/
// https://crates.io/crates/louds-rs

use crate::alphabets;
use crate::Alphabet;
use crate::LzwSpec;
use std::collections::HashMap;

// TODO implement own trie with hashtable
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

#[derive(Copy, Clone)]
pub struct Code {
    code: i32,
    used_bits: u8, // how many bits of the 32 bit integer actually constitute the code
}

impl Code {
    pub fn new() -> Code {
        // TODO replace
        Code {
            code: 0,
            used_bits: 0,
        }
    }
}

pub struct TrieNode {
    key: Option<char>,
    value: Option<Code>,
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
    pub fn new(key: char, sequence_code: Code, terminator: bool) -> TrieNode {
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

    pub fn add_child(&mut self, val: &char, sequence_code: Code, terminator: bool) {
        self.children
            .insert(*val, TrieNode::new(*val, sequence_code, terminator));
    }
}

impl TrieDictionary {
    // really we want a reference to an iterator
    // or make the struct more stateful and expose a step(char) or similar
    pub fn fetch_code_and_insert(&mut self, search_seq: &[char], next_code: Code) -> Code {
        let mut current_node = &mut self.root;
        let mut fetched_code: Option<Code> = Option::None;
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

    pub fn new(lzw_spec: LzwSpec) -> TrieDictionary {
        let mut new_trie = TrieDictionary {
            root: TrieNode::new_root(),
            // alphabet,
            clear_code: lzw_spec.clear_code,
            end_code: lzw_spec.end_code,
        };
        let alphabet: Vec<char> = alphabets::produce_alphabet(lzw_spec.alphabet);
        //
        for symbol in alphabet.iter() {
            let code = Code::new();
            new_trie.root.add_child(symbol, code, true)
        }
        new_trie
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn build_starting_dictionary() -> TrieDictionary {
        let lzw_spec: LzwSpec = LzwSpec {
            alphabet: Alphabet::Ascii,
            variable_width: false,
            min_width: 12,
            max_width: 12,
            end_code: true,
            clear_code: true,
            pack_msb_first: true,
            early_change: false,
        };
        TrieDictionary::new(lzw_spec)
    }

    #[test]
    fn fetch_existing_code() {
        let mut dict = build_starting_dictionary();

        let fetched_code = dict.fetch_code_and_insert(
            &['a', 'b'],
            Code {
                code: 1,
                used_bits: 12,
            },
        );
        assert_eq!(fetched_code.code, 0);
        let inserted_and_fetched_code = dict.fetch_code_and_insert(
            &['a', 'b', 'c'],
            Code {
                code: 2,
                used_bits: 12,
            },
        );
        assert_eq!(inserted_and_fetched_code.code, 1);
        assert_eq!(inserted_and_fetched_code.used_bits, 12);
    }
}
