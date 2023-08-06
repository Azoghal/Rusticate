use clap::Parser;
use std::collections::HashMap;
use std::env;
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

    // make root and populate with lower case dictionary
    let mut root = TrieNode::new(None, None);
    for (i, c) in "abcde".chars().enumerate() {
        root.insert(c.to_string().chars(), i.to_string());
    }
    tracing::info!("Root node after alphabet: {:?}", root);

    let key_sequence = "ababc".chars();

    let res = root.lzw_insert(key_sequence, String::from("code1"));

    tracing::info!("Root node after alphabet: {:?}", root);

    match res {
        Ok(Some(s)) => {
            tracing::info!("Got a valuable string back! {}", s);
            Ok(())
        }
        Ok(None) => Err(TrieError::Lzw("something went wrong".to_string())),
        Err(e) => Err(e),
    }

    // Ok(())
}

// TODO test mutable iterator stuff

fn mutable_it_test<I>(mut keys: I)
where
    I: Iterator<Item = char>,
{
    keys.next();
}

trait LzwDict {}

#[derive(Debug)]
enum TrieError {
    Search(String),
    Insert(String),
    Lzw(String),
}

#[derive(Debug)]
struct TrieNode {
    key: Option<char>,
    value: Option<String>,
    children: HashMap<char, TrieNode>,
}

// TODO:: benchmark the two approaches for speed.
// TODO:: Implement the LZW functionality.
// TODO:: Migrate to generic hashable tokens

impl TrieNode {
    pub fn new(key: Option<char>, value: Option<String>) -> TrieNode {
        tracing::info!("Creating a new trienode. Key:{:?} Val:{:?}", key, value);
        TrieNode {
            key,
            value,
            children: HashMap::new(),
        }
    }

    pub fn new_tail<I>(mut keys: I, value: String) -> TrieNode
    where
        I: Iterator<Item = char>,
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

    pub fn insert<I>(&mut self, mut key_it: I, value: String) -> Result<(), TrieError>
    where
        I: Iterator<Item = char>,
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
                    tracing::info!("Insert sequence exhausted, entry already in trie. Updating value of node\n{} -> {}",val, value);
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

    pub fn search<I>(&self, mut key_it: I) -> Result<Option<String>, TrieError>
    where
        I: Iterator<Item = char>,
    {
        if let Some(key) = key_it.next() {
            if let Some(node) = self.children.get(&key) {
                node.search(key_it)
            } else {
                tracing::error!(
                    "Searched for a sequence not present - does the key match? {:?} {}",
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
                    Ok(Some(val.to_string()))
                }
                None => {
                    tracing::info!("Searched for sequence has no value");
                    Ok(None)
                }
            }
        }
    }

    pub fn lzw_insert<I>(
        &mut self,
        mut key_it: I,
        new_val: String,
    ) -> Result<Option<String>, TrieError>
    where
        I: Iterator<Item = char>,
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
            Err(TrieError::Lzw("todo".to_string()))
        }
    }
}

fn insert_iter<I>(root: &mut TrieNode, mut key_it: I, value: String) -> Result<(), TrieError>
where
    I: Iterator<Item = char>,
{
    let mut node = root;

    let mut key = key_it.next();
    let Some(mut k) = key else{
        return Err(TrieError::Insert("Empty character sequence".to_string()))
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
                "Think we're in the correct node with key {:?}. Updating value {:?} -> {}",
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
            node.children.insert(
                k,
                TrieNode::new_tail(k.to_string().chars().chain(key_it), value),
            );
        }
    }
    Ok(())
}

fn search_iter<I>(root: &TrieNode, mut key_it: I) -> Result<Option<String>, TrieError>
where
    I: Iterator<Item = char>,
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
                "No such sequence in trie, {} not a child of current node",
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

#[cfg(test)]
mod test {
    use tracing_test::traced_test;

    use super::*;
    #[traced_test]
    #[test]
    fn test_insert() {
        let mut root = TrieNode::new(None, None);

        // Insert what we were looking for
        root.insert("a".chars(), "Hooray".to_string())
            .expect("Error in root.insert");

        tracing::debug!("{:?}", root);

        assert_eq!(
            root.children
                .entry('a')
                .or_insert(TrieNode::new(None, None))
                .value,
            Some("Hooray".to_string())
        );

        root.insert("abc".chars(), "Deeper".to_string())
            .expect("Error during insert");

        // First assert that an intermediate node with a key but no value was created
        let intermediate = root
            .children
            .entry('a')
            .or_insert(TrieNode::new(None, None))
            .children
            .entry('b')
            .or_insert(TrieNode::new(
                Some('X'),
                Some("Should't have this".to_string()),
            ));

        assert_eq!(intermediate.key, Some('b'));
        assert_eq!(intermediate.value, None);

        // Now assert that the leaf node has the correct key and value
        let leaf = intermediate.children.entry('c').or_insert(TrieNode::new(
            Some('X'),
            Some("Should't have this".to_string()),
        ));

        assert_eq!(leaf.key, Some('c'));
        assert_eq!(leaf.value, Some("Deeper".to_string()));
    }

    #[traced_test]
    #[test]
    fn test_search() {
        let mut root = TrieNode::new(None, None);
        let target_str = "MockValue";
        root.children
            .insert('a', TrieNode::new(Some('a'), Some(target_str.to_string())));

        tracing::debug!("{:?}", root);

        let searched_val = root
            .search("a".chars())
            .expect("Error during search")
            .unwrap();
        assert_eq!(searched_val, target_str.to_string());

        let target_str = "Deeper";
        let mut intermediate = TrieNode::new(Some('b'), None);
        intermediate
            .children
            .insert('c', TrieNode::new(Some('c'), Some(target_str.to_string())));

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
        assert_eq!(searched_val, target_str.to_string());
    }

    #[traced_test]
    #[test]
    fn test_insert_iter() {
        let mut root = TrieNode::new(None, None);

        // Insert what we were looking for
        insert_iter(&mut root, "a".chars(), "Hooray".to_string())
            .expect("Error during insert_iter");

        tracing::debug!("{:?}", root);

        assert_eq!(
            root.children
                .entry('a')
                .or_insert(TrieNode::new(None, None))
                .value,
            Some("Hooray".to_string())
        );

        insert_iter(&mut root, "abc".chars(), "Deeper".to_string())
            .expect("Error during insert iter");

        // First assert that an intermediate node with a key but no value was created
        let intermediate = root
            .children
            .entry('a')
            .or_insert(TrieNode::new(None, None))
            .children
            .entry('b')
            .or_insert(TrieNode::new(
                Some('X'),
                Some("Should't have this".to_string()),
            ));

        assert_eq!(intermediate.key, Some('b'));
        assert_eq!(intermediate.value, None);

        // Now assert that the leaf node has the correct key and value
        let leaf = intermediate.children.entry('c').or_insert(TrieNode::new(
            Some('X'),
            Some("Should't have this".to_string()),
        ));

        assert_eq!(leaf.key, Some('c'));
        assert_eq!(leaf.value, Some("Deeper".to_string()));
    }

    #[traced_test]
    #[test]
    fn test_search_iter() {
        let mut root = TrieNode::new(None, None);
        let target_str = "MockValue";
        root.children
            .insert('a', TrieNode::new(Some('a'), Some(target_str.to_string())));

        tracing::debug!("{:?}", root);

        let searched_val = search_iter(&root, "a".chars())
            .expect("Error during search")
            .unwrap();
        assert_eq!(searched_val, target_str.to_string());

        let target_str = "Deeper";
        let mut intermediate = TrieNode::new(Some('b'), None);
        intermediate
            .children
            .insert('c', TrieNode::new(Some('c'), Some(target_str.to_string())));

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
        assert_eq!(searched_val, target_str.to_string());
    }

    #[traced_test]
    #[test]
    #[should_panic]
    fn test_new_tail_empty() {
        TrieNode::new_tail("".chars(), "TheEnd".to_string());
    }

    #[traced_test]
    #[test]
    fn test_new_tail_single() {
        let target_str = "TheStartAndEnd";
        let node = TrieNode::new_tail("a".chars(), target_str.to_string());
        assert_eq!(node.key, Some('a'));
        assert_eq!(node.value, Some(target_str.to_string()));

        assert!(node.children.is_empty());
    }

    #[traced_test]
    #[test]
    fn test_new_tail_multiple() {
        let target_str = "TheEnd";
        let node = TrieNode::new_tail("abc".chars(), target_str.to_string());
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
        assert_eq!(node.value, Some(target_str.to_string()));
        assert!(node.children.is_empty());
    }
}
