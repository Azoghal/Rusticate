use criterion::{criterion_group, criterion_main, Criterion};

use mutable_trie::{IterTrie, Trie, TrieNode};

// TODO:: replace char and String with generics so can be tested with other things

fn trie(to_insert: Vec<String>, to_search: Vec<String>) {
    // insert all and then search
    let mut root = TrieNode::new(None, None);
    for (i, val) in to_insert.iter().enumerate() {
        root.insert(val.chars(), i).unwrap();
    }
    for val in to_search {
        // Todo: if searching for things not present, need to match rather than unwrap
        root.search(val.chars()).unwrap();
    }
}

fn iter_trie(to_insert: Vec<String>, to_search: Vec<String>) {
    let mut root = TrieNode::new(None, None);
    for (i, val) in to_insert.iter().enumerate() {
        TrieNode::insert_iter(&mut root, val.chars(), i).unwrap();
    }
    for val in to_search {
        // Todo: if searching for things not present, need to match rather than unwrap
        TrieNode::search_iter(&root, val.chars()).unwrap();
    }
}

fn bench_tries(c: &mut Criterion) {
    let mut group_trie: criterion::BenchmarkGroup<'_, criterion::measurement::WallTime> =
        c.benchmark_group("Trie Things");

    let to_insert: Vec<String> = include_str!("sequences.in")
        .split('\n')
        .map(String::from)
        .collect();
    let to_search = to_insert.clone();

    group_trie.bench_function("Recursive TrieNode", |b| {
        b.iter(|| trie(to_insert.clone(), to_search.clone()))
    });
    group_trie.bench_function("Iterative TrieNode", |b| {
        b.iter(|| iter_trie(to_insert.clone(), to_search.clone()))
    });

    group_trie.finish();
}

criterion_group!(benches, bench_tries);
criterion_main!(benches);
