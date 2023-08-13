use std::cmp::{Eq, PartialEq};
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::iter::{self, Peekable};
use tracing::{info, Level};

pub trait TrieKey: Copy + Hash + Debug + Eq + PartialEq {}
impl<T: Copy + Hash + Debug + Eq + PartialEq> TrieKey for T {}

pub trait TrieVal: Copy + Debug {}
impl<T: Copy + Debug> TrieVal for T {}

pub trait Trie<K, V> {
    fn insert<I: Iterator<Item = K>>(&mut self, key_it: I, value: V) -> Result<(), TrieError>;

    fn search<I: Iterator<Item = K>>(&self, key_it: I) -> Result<Option<V>, TrieError>;

    fn populate_initial<I: Iterator<Item = (K, V)>>(
        &mut self,
        kv_pair_it: I,
    ) -> Result<(), TrieError> {
        for (k, v) in kv_pair_it {
            self.insert(iter::once(k), v)?;
        }
        Ok(())
    }
}

pub trait IterTrie<T, K, V> {
    fn insert_iter<I: Iterator<Item = K>>(
        root: &mut T,
        key_it: I,
        value: V,
    ) -> Result<(), TrieError>;

    fn search_iter<I: Iterator<Item = K>>(root: &T, key_it: I) -> Result<Option<V>, TrieError>;
}

pub trait LzwDict<K, V> {
    fn lzw_insert<I: Iterator<Item = K>, J: Iterator<Item = V>>(
        &mut self,
        key_it: &mut Peekable<I>,
        new_val: &mut J,
    ) -> Result<Option<V>, TrieError>;
}

pub trait IterLzwDict<T, K, V> {
    fn lzw_insert_iter<I: Iterator<Item = K>>(
        root: &mut T,
        key_it: &mut Peekable<I>,
        value: V,
    ) -> Result<Option<V>, TrieError>;
}

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum Token<V: TrieKey> {
    End,
    Clear,
    Value(V),
}

#[derive(Debug)]
pub enum TrieError {
    Search(String),
    Insert(String),
    Lzw(String),
}

#[derive(Debug)]
pub struct TrieNode<K, V>
where
    K: TrieKey,
    V: TrieVal,
{
    key: Option<K>,
    value: Option<V>,
    children: HashMap<K, TrieNode<K, V>>,
}

impl<K, V> TrieNode<K, V>
where
    K: TrieKey,
    V: TrieVal,
{
    pub fn new(key: Option<K>, value: Option<V>) -> TrieNode<K, V> {
        //tracing::info!("Creating a new trienode. Key:{:?} Val:{:?}", key, value);
        TrieNode {
            key,
            value,
            children: HashMap::new(),
        }
    }

    pub fn new_tail<I>(mut keys: I, value: V) -> TrieNode<K, V>
    where
        I: Iterator<Item = K>,
    {
        let Some(first) = keys.next() else{
            panic!("Tail of TrieNodes cannot be created from an empty sequence");
        };
        let mut top_node = TrieNode::new(Some(first), None);
        let mut last_node = &mut top_node;
        for key in keys {
            last_node = { last_node }
                .children
                .entry(key)
                .or_insert(TrieNode::new(Some(key), None));
        }
        last_node.value = Some(value);
        top_node
    }
}

impl<K, V> Trie<K, V> for TrieNode<K, V>
where
    K: TrieKey,
    V: TrieVal,
{
    fn insert<I>(&mut self, mut key_it: I, value: V) -> Result<(), TrieError>
    where
        I: Iterator<Item = K>,
    {
        if let Some(key) = key_it.next() {
            let node = self
                .children
                .entry(key)
                .or_insert(TrieNode::new(Some(key), None));
            node.insert(key_it, value)
        } else {
            match &self.value {
                Some(val) => {
                    tracing::info!("Insert sequence exhausted, entry already in trie. Updating value of node\n{:?} -> {:?}",val, value);
                    self.value = Some(value);
                    Ok(())
                }
                None => {
                    self.value = Some(value);
                    tracing::info!("Inserting new value to trie, {:?}", self);
                    Ok(())
                }
            }
        }
    }

    fn search<I>(&self, mut key_it: I) -> Result<Option<V>, TrieError>
    where
        I: Iterator<Item = K>,
    {
        if let Some(key) = key_it.next() {
            if let Some(node) = self.children.get(&key) {
                node.search(key_it)
            } else {
                tracing::error!("Searched for a sequence not present");
                Err(TrieError::Search(
                    "Searched for sequence not present".to_string(),
                ))
            }
        } else {
            match &self.value {
                Some(val) => {
                    // tracing::info!("searched for sequence has value: {}", val);
                    Ok(Some(*val))
                }
                None => {
                    tracing::info!("Searched for sequence has no value");
                    Ok(None)
                }
            }
        }
    }
}

impl<K, V> IterTrie<TrieNode<K, V>, K, V> for TrieNode<K, V>
where
    K: TrieKey,
    V: TrieVal,
{
    fn insert_iter<I>(root: &mut TrieNode<K, V>, mut key_it: I, value: V) -> Result<(), TrieError>
    where
        I: Iterator<Item = K>,
    {
        let mut node = root;

        let mut key = key_it.next();
        let Some(mut k) = key else{
            return Err(TrieError::Insert("Empty token sequence".to_string()))
        };
        while node.children.contains_key(&k) {
            node = { node }
                .children
                .get_mut(&k)
                .expect("child corresponding to contained key not found.");
            key = key_it.next();
            match key {
                Some(new_k) => k = new_k,
                None => break, // Here we know we've reached end - just insert...
            }
        }
        // Either we're in the node that the value is destined for (None), or we need to make a path to a new node
        match key {
            None => {
                tracing::info!(
                    "Think we're in the correct node with key {:?}. Updating value {:?} -> {:?}",
                    node.key,
                    node.value,
                    value
                );
                node.value = Some(value)
            }
            Some(k) => {
                tracing::info!(
                    "Required path not in trie, making tail starting at node with key {:?}",
                    node.key
                );
                node.children
                    .insert(k, TrieNode::new_tail(iter::once(k).chain(key_it), value));
            }
        }
        Ok(())
    }

    fn search_iter<I>(root: &TrieNode<K, V>, mut key_it: I) -> Result<Option<V>, TrieError>
    where
        I: Iterator<Item = K>,
    {
        // descend
        let mut node = root;

        let mut key = key_it.next();
        let Some(mut k) = key else{
            return Err(TrieError::Search("No search sequence".to_string()));
        };
        while node.children.contains_key(&k) {
            node = node.children.get(&k).unwrap();
            key = key_it.next();
            match key {
                Some(new_k) => k = new_k,
                None => break,
            }
        }
        match key {
            Some(more_k) => {
                tracing::info!(
                    "No such sequence in trie, {:?} not a child of current node",
                    more_k
                );
                Ok(None)
            }
            None => {
                tracing::info!("found the value");
                Ok(node.value)
            }
        }
    }
}

impl<K, V> LzwDict<K, V> for TrieNode<K, V>
where
    K: TrieKey,
    V: TrieVal,
{
    fn lzw_insert<I, J>(
        &mut self,
        key_it: &mut Peekable<I>,
        new_val: &mut J,
    ) -> Result<Option<V>, TrieError>
    where
        I: Iterator<Item = K>,
        J: Iterator<Item = V>,
    {
        // Peek at the next item
        if let Some(key) = key_it.peek() {
            if let Some(node) = self.children.get_mut(key) {
                // step down the trie into this node and continue consuming tokens
                key_it.next(); // advance iterator
                let inner_result = node.lzw_insert(key_it, new_val)?;
                Ok(inner_result)
            } else {
                // we are currently in the last node on the iterator's path through the trie
                // create a new node and add to children
                // DO NOT advance the iterator, as next call needs to start at this key
                // return the stored value
                self.children.insert(
                    *key,
                    TrieNode::new(Some(*key), Some(new_val.next().unwrap())),
                );
                Ok(self.value)
            }
        } else {
            //TODO: this isn't sufficient to cope with a valid end of stream... unless always a certain end token?
            info!(
                "Escaping lzw_insert due to end of sequence, trying to add code from empty iterator"
            );
            Err(TrieError::Lzw(
                "Empty search sequence before new node created".to_string(),
            ))
        }
    }
}

impl<K, V> IterLzwDict<TrieNode<K, V>, K, V> for TrieNode<K, V>
where
    K: TrieKey,
    V: TrieVal,
{
    fn lzw_insert_iter<I>(
        root: &mut TrieNode<K, V>,
        key_it: &mut Peekable<I>,
        value: V,
    ) -> Result<Option<V>, TrieError>
    where
        I: Iterator<Item = K>,
    {
        // Descend down the nodes until not contained as a child
        // Insert new child, return value
        let mut node = root;

        while let Some(k) = key_it.peek() {
            if node.children.contains_key(k) {
                //let _ = key_it.next(); // Advance the iterator after peeking at a contained key
                node = { node }
                    .children
                    .get_mut(k)
                    .expect("child corresponding to contained key not found.");
                key_it.next(); // advance the iterator
            } else {
                // Reached a node where the children does not contain k
                node.children
                    .insert(*k, TrieNode::new(Some(*k), Some(value)));
                return Ok(node.value);
            }
        }
        // TODO return the code without adding new sequence
        Err(TrieError::Lzw("Reached end of sequence".to_string()))
    }
}

// TODO: refactor lzw methods to use Peekable and therefore leave iterator in correct state.

#[cfg(test)]
mod test {
    use tracing_test::traced_test;

    use super::*;
    #[traced_test]
    #[test]
    fn test_insert() {
        let mut root = TrieNode::new(None, None);

        // Insert what we were looking for
        root.insert("a".chars(), "Hooray")
            .expect("Error in root.insert");

        tracing::debug!("{:?}", root);

        assert_eq!(
            root.children
                .entry('a')
                .or_insert(TrieNode::new(None, None))
                .value,
            Some("Hooray")
        );

        root.insert("abc".chars(), "Deeper")
            .expect("Error during insert");

        // First assert that an intermediate node with a key but no value was created
        let intermediate = root
            .children
            .entry('a')
            .or_insert(TrieNode::new(None, None))
            .children
            .entry('b')
            .or_insert(TrieNode::new(Some('X'), Some("Should't have this")));

        assert_eq!(intermediate.key, Some('b'));
        assert_eq!(intermediate.value, None);

        // Now assert that the leaf node has the correct key and value
        let leaf = intermediate
            .children
            .entry('c')
            .or_insert(TrieNode::new(Some('X'), Some("Should't have this")));

        assert_eq!(leaf.key, Some('c'));
        assert_eq!(leaf.value, Some("Deeper"));
    }

    #[traced_test]
    #[test]
    fn test_search() {
        let mut root = TrieNode::new(None, None);
        let target_str = "MockValue";
        root.children
            .insert('a', TrieNode::new(Some('a'), Some(target_str)));

        tracing::debug!("{:?}", root);

        let searched_val = root
            .search("a".chars())
            .expect("Error during search")
            .unwrap();
        assert_eq!(searched_val, target_str);

        let target_str = "Deeper";
        let mut intermediate = TrieNode::new(Some('b'), None);
        intermediate
            .children
            .insert('c', TrieNode::new(Some('c'), Some(target_str)));

        root.children
            .entry('a')
            .or_insert(TrieNode::new(None, None))
            .children
            .insert('b', intermediate);

        let searched_val = root.search("ab".chars()).expect("Error during search");
        assert_eq!(searched_val, None);

        let searched_val = root
            .search("abc".chars())
            .expect("Error during search")
            .unwrap();
        assert_eq!(searched_val, target_str);
    }

    #[traced_test]
    #[test]
    fn test_insert_iter() {
        let mut root = TrieNode::new(None, None);

        // Insert what we were looking for
        TrieNode::insert_iter(&mut root, "a".chars(), "Hooray").expect("Error during insert_iter");

        tracing::debug!("{:?}", root);

        assert_eq!(
            root.children
                .entry('a')
                .or_insert(TrieNode::new(None, None))
                .value,
            Some("Hooray")
        );

        TrieNode::insert_iter(&mut root, "abc".chars(), "Deeper")
            .expect("Error during insert iter");

        // First assert that an intermediate node with a key but no value was created
        let intermediate = root
            .children
            .entry('a')
            .or_insert(TrieNode::new(None, None))
            .children
            .entry('b')
            .or_insert(TrieNode::new(Some('X'), Some("Should't have this")));

        assert_eq!(intermediate.key, Some('b'));
        assert_eq!(intermediate.value, None);

        // Now assert that the leaf node has the correct key and value
        let leaf = intermediate
            .children
            .entry('c')
            .or_insert(TrieNode::new(Some('X'), Some("Should't have this")));

        assert_eq!(leaf.key, Some('c'));
        assert_eq!(leaf.value, Some("Deeper"));
    }

    #[traced_test]
    #[test]
    fn test_search_iter() {
        let mut root = TrieNode::new(None, None);
        let target_str = "MockValue";
        root.children
            .insert('a', TrieNode::new(Some('a'), Some(target_str)));

        tracing::debug!("{:?}", root);

        let searched_val = TrieNode::search_iter(&root, "a".chars())
            .expect("Error during search")
            .unwrap();
        assert_eq!(searched_val, target_str);

        let target_str = "Deeper";
        let mut intermediate = TrieNode::new(Some('b'), None);
        intermediate
            .children
            .insert('c', TrieNode::new(Some('c'), Some(target_str)));

        root.children
            .entry('a')
            .or_insert(TrieNode::new(None, None))
            .children
            .insert('b', intermediate);

        let searched_val = TrieNode::search_iter(&root, "ab".chars()).expect("Error during search");
        assert_eq!(searched_val, None);

        let searched_val = TrieNode::search_iter(&root, "abc".chars())
            .expect("Error during search")
            .unwrap();
        assert_eq!(searched_val, target_str);
    }

    #[traced_test]
    #[test]
    fn test_lzw_insert() {
        // make root and populate with 5 lower case dictionary
        let mut root: TrieNode<char, usize> = TrieNode::new(None, None);
        for (i, c) in "abcde".chars().enumerate() {
            root.insert(iter::once(c), i).unwrap();
        }
        tracing::info!("Root node after alphabet: {:?}", root);

        let mut key_sequence = "ababc".chars().peekable();
        let mut val_sequence = vec![99, 100, 101].into_iter();

        // Insert sequence "ab" and recieve the code for sequence "a"
        let Ok(Some(val)) = root.lzw_insert(&mut key_sequence, &mut val_sequence) else{
            panic!("expected to recieve a value from lzw_insert");
        };
        assert_eq!(val, 0);

        // Insert the sequence "ba" and recieve the code for sequence "b"
        let Ok(Some(val)) = root.lzw_insert(&mut key_sequence, &mut val_sequence) else{
            panic!("expected to recieve a value from lzw_insert");
        };
        assert_eq!(val, 1);

        // Insert the sequence "abc" and recieve the code for sequence "ab"
        let Ok(Some(val)) = root.lzw_insert(&mut key_sequence, &mut val_sequence) else{
            panic!("expected to recieve a value from lzw_insert");
        };
        assert_eq!(val, 99);

        // Assert that the iterator still contains "c" - ready for the next insert
        let Some(c) = key_sequence.next() else{
            panic!();
        };
        assert_eq!(c, 'c');

        // Use search method to find the value in second inserted sequence
        let Ok(Some(val)) = root.search("ba".chars()) else{
            panic!("expected to recieve a value from lzw_insert");
        };
        assert_eq!(val, 100);

        // Use search method to find the value in second inserted sequence
        let Ok(Some(val)) = root.search("abc".chars()) else{
            panic!("expected to recieve a value from lzw_insert");
        };
        assert_eq!(val, 101);
    }

    #[traced_test]
    #[test]
    fn test_lzw_insert_iter() {
        // make root and populate with 5 lower case dictionary
        let mut root = TrieNode::new(None, None);
        for (i, c) in "abcde".chars().enumerate() {
            root.insert(iter::once(c), i).unwrap();
        }
        tracing::info!("Root node after alphabet: {:?}", root);

        let mut key_sequence = "ababc".chars().peekable();

        // Insert sequence "ab" and recieve the code for sequence "a"
        let Ok(Some(val)) = TrieNode::lzw_insert_iter(&mut root, &mut key_sequence, 99) else{
            panic!("expected to recieve a value from lzw_insert");
        };
        assert_eq!(val, 0);

        // Insert sequence "ba" and recieve the code for sequence "b"
        let Ok(Some(val)) = TrieNode::lzw_insert_iter(&mut root, &mut key_sequence, 100) else{
            panic!("expected to recieve a value from lzw_insert");
        };
        assert_eq!(val, 1);

        // Insert the remaining sequence "abc" and recieve the code for sequence "ab"
        let Ok(Some(val)) = TrieNode::lzw_insert_iter(&mut root, &mut key_sequence, 101) else{
            panic!("expected to recieve a value from lzw_insert");
        };
        assert_eq!(val, 99);

        // Assert that the iterator still contains "c" - ready for the next insert
        let Some(c) = key_sequence.next() else{
            panic!();
        };
        assert_eq!(c, 'c');

        // Use search method to find the value in second inserted sequence
        let Ok(Some(val)) = root.search("abc".chars()) else{
            panic!("expected to recieve a value from lzw_insert");
        };
        assert_eq!(val, 101);
    }

    #[traced_test]
    #[test]
    fn test_token() {
        let mut root = TrieNode::new(None, None);
        root.insert(iter::once(Token::Value('a')), 0).unwrap();
        root.insert(iter::once(Token::Value('b')), 1).unwrap();
        root.insert(iter::once(Token::Value('c')), 2).unwrap();
        root.insert(iter::once(Token::Value('d')), 3).unwrap();
        root.insert(iter::once(Token::Value('e')), 4).unwrap();

        root.insert(iter::once(Token::Clear), 5).unwrap();
        root.insert(iter::once(Token::End), 6).unwrap();
        tracing::info!("Root node after alphabet: {:?}", root);

        let mut key_sequence = vec![
            Token::Value('a'),
            Token::Value('b'),
            Token::Value('a'),
            Token::Value('b'),
            Token::Value('c'),
        ]
        .into_iter()
        .peekable();
        let mut val_sequence = vec![99, 100, 101].into_iter();

        // Insert sequence "ab"=99 and recieve the code for sequence "a"
        let Ok(Some(val)) = root.lzw_insert(&mut key_sequence, &mut val_sequence) else{
            panic!("expected to recieve a value from lzw_insert");
        };
        assert_eq!(val, 0);

        // Insert sequence "ba"=100 and recieve the code for sequence "b"
        let Ok(Some(val)) = root.lzw_insert(&mut key_sequence, &mut val_sequence) else{
            panic!("expected to recieve a value from lzw_insert");
        };
        assert_eq!(val, 1);

        // Insert the remaining sequence "abc"=101 and recieve the code for sequence "ab"
        let Ok(Some(val)) = root.lzw_insert(&mut key_sequence, &mut val_sequence) else{
            panic!("expected to recieve a value from lzw_insert");
        };
        assert_eq!(val, 99);

        let final_search_sequence =
            vec![Token::Value('a'), Token::Value('b'), Token::Value('c')].into_iter();
        // Use search method to find the value in longest inserted sequence
        let Ok(Some(val)) = root.search(final_search_sequence) else{
            panic!("expected to recieve a value from lzw_insert");
        };
        assert_eq!(val, 101);
    }

    #[traced_test]
    #[test]
    fn test_wikipedia_input() {
        let to_insert = String::from("tobeornottobeortobeornot");
        let mut root: TrieNode<char, i32> = TrieNode::new(None, None);

        let alpha_codes = "abcdefghijklmnopqrstuvwxyz"
            .char_indices()
            .map(|(u, c)| (c, u as i32));

        root.populate_initial(alpha_codes);
        let mut char_iter = to_insert.chars().peekable();
        let mut codes = 26..;
        while let Ok(Some(_v)) = root.lzw_insert(&mut char_iter, &mut codes) {}

        let Some(next_code) = codes.next() else{
            panic!("out of codes");
        };
        // 0-40 used, 41 should be the next
        assert_eq!(next_code, 41);

        let trie_paths = (vec![
            "be", "beo", "eo", "eor", "no", "ob", "or", "ort", "ot", "rn", "rno", "to", "tob",
            "tobe", "tt",
        ])
        .into_iter();
        let trie_vals =
            (vec![28, 36, 29, 39, 32, 27, 30, 37, 33, 31, 40, 26, 35, 38, 34]).into_iter();
        assert_eq!(trie_paths.len(), trie_vals.len());
        let path_vals = trie_paths.zip(trie_vals);

        for (path, exp_val) in path_vals {
            let found_val = root.search(path.chars()).unwrap().unwrap();
            assert_eq!(found_val, exp_val);
        }
    }

    #[traced_test]
    #[test]
    fn test_populate_initial() {
        let alphabet = "abcdefghijklmnopqrstuvwxyz".chars();
        let codes = 0..26;
        let alpha_codes = alphabet.zip(codes);
        let mut root: TrieNode<char, i32> = TrieNode::new(None, None);
        root.populate_initial(alpha_codes).unwrap();

        let a = root.children.get(&'a').unwrap().value;
        assert_eq!(a, Some(0));

        let z = root.children.get(&'z').unwrap().value;
        assert_eq!(z, Some(25));
    }

    #[traced_test]
    #[test]
    #[should_panic]
    fn test_new_tail_empty() {
        TrieNode::new_tail("".chars(), "TheEnd");
    }

    #[traced_test]
    #[test]
    fn test_new_tail_single() {
        let target_str = "TheStartAndEnd";
        let node = TrieNode::new_tail("a".chars(), target_str);
        assert_eq!(node.key, Some('a'));
        assert_eq!(node.value, Some(target_str));

        assert!(node.children.is_empty());
    }

    #[traced_test]
    #[test]
    fn test_new_tail_multiple() {
        let target_str = "TheEnd";
        let node = TrieNode::new_tail("abc".chars(), target_str);
        assert_eq!(node.key, Some('a'));
        assert_eq!(node.value, None);

        let Some(node) = node.children.get(&'b') else{
            panic!("Expected intermediate node at 'b'");
        };
        assert_eq!(node.key, Some('b'));
        assert_eq!(node.value, None);

        let Some(node) = node.children.get(&'c') else{
            panic!("Expected leaf node at 'c'");
        };
        assert_eq!(node.key, Some('c'));
        assert_eq!(node.value, Some(target_str));
        assert!(node.children.is_empty());
    }
}
