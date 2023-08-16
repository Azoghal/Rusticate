use criterion::{criterion_group, criterion_main, Criterion};

use lzw::lzw_dict::{IterLzwDict, LzwDict};
use mutable_trie::{Trie, TrieNode};

fn lzw_trie(to_insert: String) {
    let alpha_codes = "abcdefghijklmnopqrstuvwxyz"
        .char_indices()
        .map(|(u, c)| (c, u as i32));

    let mut root: TrieNode<char, i32> = TrieNode::new(None, None);
    root.populate_initial(alpha_codes).unwrap();

    let mut char_iter = to_insert.chars().peekable();
    let mut code = 26..;
}

fn iter_lzw_trie(to_insert: String) {
    let alpha_codes = "abcdefghijklmnopqrstuvwxyz"
        .char_indices()
        .map(|(u, c)| (c, u as i32));

    let mut root: TrieNode<char, i32> = TrieNode::new(None, None);
    root.populate_initial(alpha_codes).unwrap();

    let mut char_iter = to_insert.chars().peekable();
    // we need to make insert calls while char_iter is not empty
    let mut code = 26;
    while let Ok(Some(_)) = TrieNode::lzw_insert_iter(&mut root, &mut char_iter, code) {
        code += 1;
    }
}

fn bench_lzw_tries(c: &mut Criterion) {
    let mut group_lzw: criterion::BenchmarkGroup<'_, criterion::measurement::WallTime> =
        c.benchmark_group("LZW Things");

    let to_insert: &str = include_str!("lzw_sequences.in");

    group_lzw.bench_function("Recursive LzwDict", |b| {
        b.iter(|| lzw_trie(String::from(to_insert)))
    });
    group_lzw.bench_function("Iterative LzwDict", |b| {
        b.iter(|| iter_lzw_trie(String::from(to_insert)))
    });

    group_lzw.finish();
}

criterion_group!(benches, bench_lzw_tries);
criterion_main!(benches);
