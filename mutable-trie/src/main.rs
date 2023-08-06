use clap::Parser;
use std::cmp::{Eq, PartialEq};
use std::collections::HashMap;
use std::env;
use std::fmt::Debug;
use std::hash::Hash;
use std::iter;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Parser)]
#[command(author, version, about, long_about=None)]
// #[command(propagate_version = true)]
struct Args {
    #[arg(default_value_t = 10)]
    num_nodes: u8,
}

fn main() -> Result<(), TrieError> {
    env::set_var("RUST_BACKTRACE", "1");
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let args = Args::parse();
    info!(
        "Starting to make a trie with the following number of nodes: {}",
        args.num_nodes
    );

    Ok(())
}

trait LzwDict {}

#[derive(Copy, Clone, Debug, Hash)]
enum Token<V: Copy + Clone + Debug + Hash> {
    End,
    Clear,
    Value(V),
}

#[derive(Debug)]
enum TrieError {
    Search(String),
    Insert(String),
    Lzw(String),
}

#[derive(Debug)]
struct TrieNode<K, V>
where
    K: Copy + Hash + Debug + Eq + PartialEq,
    V: Copy + Debug,
{
    key: Option<K>,
    value: Option<V>,
    children: HashMap<K, TrieNode<K, V>>,
}

// TODO:: benchmark the two approaches for speed.
// TODO:: Migrate to generic hashable tokens
// TODO:: Migrate insert and search to be a trait
// TODO:: Migrate lzw_insert to be a trait

impl<K, V> TrieNode<K, V>
where
    K: Copy + Hash + Debug + Eq + PartialEq,
    V: Copy + Debug,
{
    pub fn new(key: Option<K>, value: Option<V>) -> TrieNode<K, V> {
        tracing::info!("Creating a new trienode. Key:{:?} Val:{:?}", key, value);
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

    pub fn insert<I>(&mut self, mut key_it: I, value: V) -> Result<(), TrieError>
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

    pub fn search<I>(&self, mut key_it: I) -> Result<Option<V>, TrieError>
    where
        I: Iterator<Item = K>,
    {
        if let Some(key) = key_it.next() {
            if let Some(node) = self.children.get(&key) {
                node.search(key_it)
            } else {
                tracing::error!(
                    "Searched for a sequence not present - does the key match? {:?} {:?}",
                    self.key,
                    key
                );
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

    pub fn lzw_insert<I>(&mut self, mut key_it: I, new_val: V) -> Result<Option<V>, TrieError>
    where
        I: Iterator<Item = K>,
    {
        // new_val is the code
        // use the iterator to go down the trie
        // remembe the last code found
        // insert a single extra symbol and return the last code
        if let Some(key) = key_it.next() {
            if let Some(node) = self.children.get_mut(&key) {
                // step down the trie into this node and continue consuming tokens
                let inner_result = node.lzw_insert(key_it, new_val)?;
                Ok(inner_result)
            } else {
                // we are currently in the last node on the iterator path.
                // create a new node and add to children
                // return the stored value
                self.children
                    .insert(key, TrieNode::new(Some(key), Some(new_val)));
                Ok(self.value.clone())
            }
        } else {
            //TODO: this isn't sufficient to cope with a valid end of stream... unless always a certain end token?
            Err(TrieError::Lzw(
                "Empty search sequence before new node created".to_string(),
            ))
        }
    }
}

fn insert_iter<K, V, I>(root: &mut TrieNode<K, V>, mut key_it: I, value: V) -> Result<(), TrieError>
where
    K: Copy + Hash + Debug + Eq + PartialEq,
    V: Copy + Debug,
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

fn search_iter<K, V, I>(root: &TrieNode<K, V>, mut key_it: I) -> Result<Option<V>, TrieError>
where
    K: Copy + Hash + Debug + Eq + PartialEq,
    V: Copy + Debug,
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
            Ok(node.value.clone())
        }
    }
}

fn lzw_insert_iter<K, V, I>(
    root: &mut TrieNode<K, V>,
    mut key_it: I,
    value: V,
) -> Result<Option<V>, TrieError>
where
    K: Hash + Copy + Debug + Eq + PartialEq,
    V: Copy + Debug,
    I: Iterator<Item = K>,
{
    // Descend down the nodes until not contained as a child
    // Insert new child, return value
    let mut node = root;

    let mut key = key_it.next();
    let Some(mut k) = key else{
        return Err(TrieError::Lzw("Empty character sequence".to_string()))
    };
    while node.children.contains_key(&k) {
        node = { node }
            .children
            .get_mut(&k)
            .expect("child corresponding to contained key not found.");
        key = key_it.next();
        match key {
            Some(new_k) => k = new_k,
            None => {
                return Err(TrieError::Lzw(
                    "Empty character sequence before new node created".to_string(),
                ))
            }
        }
    }
    // Reached a node where the children does not contain k
    node.children.insert(k, TrieNode::new(Some(k), Some(value)));
    Ok(node.value)
}

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
        insert_iter(&mut root, "a".chars(), "Hooray").expect("Error during insert_iter");

        tracing::debug!("{:?}", root);

        assert_eq!(
            root.children
                .entry('a')
                .or_insert(TrieNode::new(None, None))
                .value,
            Some("Hooray")
        );

        insert_iter(&mut root, "abc".chars(), "Deeper").expect("Error during insert iter");

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

        let searched_val = search_iter(&root, "a".chars())
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

        let searched_val = search_iter(&root, "ab".chars()).expect("Error during search");
        assert_eq!(searched_val, None);

        let searched_val = search_iter(&root, "abc".chars())
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
            root.insert(iter::once(c), i);
        }
        tracing::info!("Root node after alphabet: {:?}", root);

        let mut key_sequence = "ababc".chars();

        // Insert sequence "ab" and recieve the code for sequence "a"
        let Ok(Some(val)) = root.lzw_insert(&mut key_sequence, 99) else{
            panic!("expected to recieve a value from lzw_insert");
        };
        assert_eq!(val, 0);

        // Insert the remaining sequence "abc" and recieve the code for sequence "ab"
        let Ok(Some(val)) = root.lzw_insert(&mut key_sequence, 100) else{
            panic!("expected to recieve a value from lzw_insert");
        };
        assert_eq!(val, 99);

        // Use search method to find the value in second inserted sequence
        let Ok(Some(val)) = root.search("abc".chars()) else{
            panic!("expected to recieve a value from lzw_insert");
        };
        assert_eq!(val, 100);
    }

    #[traced_test]
    #[test]
    fn test_lzw_insert_iter() {
        // make root and populate with 5 lower case dictionary
        let mut root = TrieNode::new(None, None);
        for (i, c) in "abcde".chars().enumerate() {
            root.insert(iter::once(c), i);
        }
        tracing::info!("Root node after alphabet: {:?}", root);

        let mut key_sequence = "ababc".chars();

        // Insert sequence "ab" and recieve the code for sequence "a"
        let Ok(Some(val)) = lzw_insert_iter(&mut root, &mut key_sequence, 99) else{
            panic!("expected to recieve a value from lzw_insert");
        };
        assert_eq!(val, 0);

        // Insert the remaining sequence "abc" and recieve the code for sequence "ab"
        let Ok(Some(val)) = lzw_insert_iter(&mut root, &mut key_sequence, 100) else{
            panic!("expected to recieve a value from lzw_insert");
        };
        assert_eq!(val, 99);

        // Use search method to find the value in second inserted sequence
        let Ok(Some(val)) = root.search("abc".chars()) else{
            panic!("expected to recieve a value from lzw_insert");
        };
        assert_eq!(val, 100);
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
