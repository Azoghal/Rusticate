use std::{
    collections::HashMap,
    iter::{self, Peekable},
};

use clap::Parser;
use itertools::{Itertools, PeekingNext, TupleWindows};
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Parser)]
#[command(author, version, about, long_about=None)]
// #[command(propagate_version = true)]
struct Args {
    #[arg(default_value_t = 10)]
    arg_num: u8,
}

fn make_peekable_and_ret<I>(mut the_it: I)
where
    I: Iterator<Item = char>,
{
    // DOESN't work
    println!("Going to next, then peek, then exit");
    println!("Next: {:?}", the_it.next());
    println!("Peeking at the thing. 👀{:?}", the_it.peekable().peek());
}

fn take_peekable_and_ret<I>(the_peekable: &mut Peekable<I>)
where
    I: Iterator<Item = char>,
{
    // DOESN't work
    println!("Going to next, then peek, then exit");
    println!("Next: {:?}", the_peekable.next());
    println!("Peeking at the thing. 👀{:?}", the_peekable.peek());
    println!("Next: {:?}", the_peekable.next());
    println!("Peeking at the thing. 👀{:?}", the_peekable.peek());
}

fn take_it_and_ret_it<'a, I>(the_it: &'a mut I) -> impl Iterator<Item = char> + 'a
where
    I: Iterator<Item = char> + 'a,
{
    println!("Going to next, then peek, then exit");
    let popped = the_it.next().unwrap();
    println!("Next was {}", popped);
    iter::once(popped).chain(the_it)
}

fn increment(a: u16) -> u16 {
    a + 1
}

// fn take_3_with_inspect_wrapper_2(mut the_iter: &Box<dyn Iterator<Item = char>>) {
//     let popped = take_3_ret_1(&mut the_iter);
//     let temp = Box::new(iter::once(popped).chain(the_iter));
// }

fn take_3_with_inspect_wrapper<'a>(
    the_iter: &'a mut Box<dyn Iterator<Item = char>>,
) -> Box<dyn Iterator<Item = char> + 'a> {
    let popped = take_3_ret_1(the_iter);
    Box::new(iter::once(popped).chain(the_iter))
}

fn take_3_ret_1(iter: &mut Box<dyn Iterator<Item = char>>) -> char {
    iter.next().unwrap();
    iter.next().unwrap();
    let val = iter.next().unwrap();
    val
}

// TODO: try some stuff with IterTools
// TODO: try with PeekingNext from IterTOols

fn with_tuple_windows<I>(it: &mut I)
where
    I: Iterator<Item = char>,
{
    let mut tupledude = it.tuple_windows::<(char, char)>();
    // for (a, b) in tupledude {
    //     println!("{a} {b}");
    // }
    if let Some((a, b)) = tupledude.next() {
        println!("{a}, {b}");
    };
}

// fn with_peeking_next<I>(it: &mut Peekable<I>)
// where
//     I: Iterator<Item = char>,
// {
//     let candidates: Vec<char> = vec!['a', 'b', 'c'];
//     //let mut peekingdude = it.peeking_next(|c| candidates.contains(c));
//     // for (a, b) in tupledude {
//     //     println!("{a} {b}");
//     // }
//     while let Some(a) = it.peeking_next(|c| candidates.contains(c)) {
//         println!("{a}");
//     }
// }

fn take_if_in_map<I>(map: HashMap<char, i32>, it: &mut Peekable<I>)
where
    I: Iterator<Item = char>,
{
    let op = it.peek();
    match op {
        Some(c) => {
            println!("{:?} ", c);
            if map.contains_key(c) {
                println!(
                    "Jump Down a node, using next to move cursor on: {:?}",
                    it.next()
                );
            } else {
                println!("make a new node!")
            }
        }
        None => println!("Empty!!!"),
    }
}

fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let _args = Args::parse();

    // let mut char_iter: Box<dyn Iterator<Item = char>> = Box::new("abcdefgh".chars());

    // for c in take_3_with_inspect_wrapper(&mut char_iter) {
    //     println!("{c}");
    // }

    // let mut map: HashMap<char, i32> = HashMap::new();
    // map.insert('a', 1);

    // let mut char_iter = "abcdefghi".chars();

    // while let Some(key) = char_iter.peeking_next(|c| map.contains_key(c)) {
    //     println!("{:?} : {:?}", key, map.get(&key));
    // }
    // println!("after: {:?}", char_iter.next());
    // let mut peekable_chars = "abcdefghi".chars().peekable();
    // take_peekable_and_ret(&mut peekable_chars);
    // println!("Nexting outside of function {:?}", peekable_chars.next());

    let mut map: HashMap<char, i32> = HashMap::new();
    // map.insert('a', 1);
    // map.insert('b', 1);
    // map.insert('c', 1);
    // map.insert('d', 1);

    let mut peekable_chars = "abcdefghi".chars().peekable();

    take_if_in_map(map, &mut peekable_chars);
    println!("{:?}", peekable_chars.next());
}
