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
    println!("Going to next, then peek, then exit");
    println!("Next: {:?}", the_it.next());
    println!("Peeking at the thing. 👀{:?}", the_it.peekable().peek());
}

fn take_peekable_and_ret<I>(the_peekable: &mut Peekable<I>)
where
    I: Iterator<Item = char>,
{
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

fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let _args = Args::parse();

    info!("Going to make an iterator, pass it as mutable to a function, which is going to make it peekable, next, peek, and then exit");
    let mut char_iter = "abcde".chars();
    make_peekable_and_ret(&mut char_iter);
    println!("What happens when i get back? Next: {:?}", char_iter.next());

    info!("Going to make a peekable iterator, pass it as mutable to a function, which is going to next, peek, and then exit");
    let mut char_iter = "abcde".chars().peekable();
    take_peekable_and_ret(&mut char_iter);
    println!("What happens when i get back? Next: {:?}", char_iter.next());

    info!("Going to make an iterator, pass it as mutable to a function, which is going to next, re-attach it and return it");
    let mut char_iter = "abcde".chars();
    let mut new_iter = take_it_and_ret_it(&mut char_iter);
    println!("What happens when i get back? Next: {:?}", new_iter.next());
}
