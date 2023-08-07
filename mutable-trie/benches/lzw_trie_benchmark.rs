use criterion::{criterion_group, criterion_main, Criterion};

use mutable_trie::{IterLzwDict, LzwDict, TrieNode};

fn lzw_trie(to_insert: Vec<String>) {
    let mut root = TrieNode::new(None, None);
    for (i, val) in to_insert.iter().enumerate() {
        root.lzw_insert(val.chars(), i).unwrap();
    }
}

fn iter_lzw_trie(to_insert: Vec<String>) {
    let mut root = TrieNode::new(None, None);
    for (i, val) in to_insert.iter().enumerate() {
        TrieNode::lzw_insert_iter(&mut root, val.chars(), i).unwrap();
    }
}

fn bench_lzw_tries(c: &mut Criterion) {
    let mut group_lzw: criterion::BenchmarkGroup<'_, criterion::measurement::WallTime> =
        c.benchmark_group("LZW Things");

    let to_insert: Vec<String> = include_str!("lzw_sequences.in")
        .split('\n')
        .map(String::from)
        .collect();

    group_lzw.bench_function("Recursive LzwDict", |b| {
        b.iter(|| lzw_trie(to_insert.clone()))
    });
    group_lzw.bench_function("Iterative LzwDict", |b| {
        b.iter(|| iter_lzw_trie(to_insert.clone()))
    });

    group_lzw.finish();
}

criterion_group!(benches, bench_lzw_tries);
criterion_main!(benches);
