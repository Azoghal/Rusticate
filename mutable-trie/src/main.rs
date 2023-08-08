use std::iter::{self, Peekable};

use clap::Parser;
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
    println!(
        "Peeking at the thing. 👀{:?}",
        the_peekable.peekable().peek()
    );
    println!("Next: {:?}", the_peekable.next());
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

fn take_3_with_inspect_wrapper_2(mut the_iter: Box<dyn Iterator<Item = char>>) {
    let popped = take_3_ret_1(&mut the_iter);
    the_iter = Box::new(iter::once(popped).chain(the_iter));
}

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

fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let _args = Args::parse();

    info!("Going to make an iterator, pass it as mutable to a function, which is going to next, re-attach it and return it");

    let mut char_iter: Box<dyn Iterator<Item = char>> = Box::new("abcdefgh".chars());

    for c in take_3_with_inspect_wrapper(&mut char_iter) {
        println!("{c}");
    }

    char_iter.next();
}
