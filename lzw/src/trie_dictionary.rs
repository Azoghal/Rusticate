// https://en.wikipedia.org/wiki/Trie
// https://docs.rs/trie-rs/latest/trie_rs/
// https://crates.io/crates/louds-rs

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
    alphabet: Vec<char>,
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

    pub fn addChild(&mut self, val: &char, sequence_code: Code, terminator: bool) {
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
        for label in search_seq.iter() {
            if current_node.children.contains_key(label) {
                current_node = current_node.children.get_mut(label).unwrap();
            } else {
                fetched_code = current_node.value;
                current_node.addChild(label, next_code, true);
                current_node.terminator = false;
            }
        }
        match fetched_code {
            Some(code) => code,
            None => panic!("didn't fetch a code"),
        }
    }
}
