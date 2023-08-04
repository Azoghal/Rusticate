use clap::Parser;
use std::collections::HashMap;
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Parser)]
#[command(author, version, about, long_about=None)]
// #[command(propagate_version = true)]
struct Args {
    #[arg(default_value_t = 10)]
    num_nodes: u8,
}

fn main() -> Result<(), TrieError> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let args = Args::parse();
    info!(
        "Starting to make a trie with the following number of nodes: {}",
        args.num_nodes
    );

    let mut root = TrieNode::new(None, None);
    let mut chars = root.insert("abc".chars(), "Hooray".to_string())?;
    tracing::info!("Successfully added");
    tracing::info!("All of chars used? {}", chars.next().is_none());
    let (res_val, chars) = root.search("ab".chars())?;
    tracing::info!(
        "Searched for ab: Res: {:?}, remaining chars: {:?}",
        res_val,
        chars
    );
    Ok(())
}

#[derive(Debug)]
enum TrieError {
    Search(String),
    Insert(String),
    Lzw(String),
}

struct TrieNode {
    key: Option<char>,
    value: Option<String>,
    children: HashMap<char, TrieNode>,
}

impl TrieNode {
    pub fn new(key: Option<char>, value: Option<String>) -> TrieNode {
        tracing::info!("Creating a new trienode. Key:{:?} Val:{:?}", key, value);
        TrieNode {
            key,
            value,
            children: HashMap::new(),
        }
    }

    pub fn insert<I>(&mut self, mut key_it: I, value: String) -> Result<I, TrieError>
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
                    Ok(key_it)
                }
                None => {
                    tracing::info!("Inserting new value to trie, {}", value);
                    self.value = Some(value);
                    Ok(key_it)
                }
            }
        }
    }

    pub fn search<I>(&self, mut key_it: I) -> Result<(Option<String>, I), TrieError>
    where
        I: Iterator<Item = char>,
    {
        if let Some(key) = key_it.next() {
            if let Some(node) = self.children.get(&key) {
                node.search(key_it)
            } else {
                Err(TrieError::Search(
                    "Searched for sequence not present".to_string(),
                ))
            }
        } else {
            match &self.value {
                Some(val) => {
                    // tracing::info!("searched for sequence has value: {}", val);
                    Ok((Some(val.to_string()), key_it))
                }
                None => {
                    tracing::info!("Searched for sequence has no value");
                    Ok((None, key_it))
                }
            }
        }
    }
}

#[cfg(test)]
mod test {}
