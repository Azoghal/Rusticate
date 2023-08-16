use mutable_trie::{TrieError, TrieKey, TrieNode, TrieVal};
use std::iter::Peekable;
use tracing::info;

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum Token<V: TrieKey> {
    End,
    Clear,
    Value(V),
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

// TODO: move this over to lzw. bring tests
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
        // TODO: replace with match for Some(End), Some(_), None? but then this isn't generic
        // Peek at the next item
        // TODO: rethink.
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

#[cfg(test)]
mod test {
    use super::*;
    use mutable_trie::Trie;
    use std::iter;
    use tracing_test::traced_test;

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

        root.populate_initial(alpha_codes).unwrap();
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
}
