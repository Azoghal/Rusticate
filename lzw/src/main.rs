use alphabets::Alphabetable;
use base64::engine::general_purpose;
use clap::{Args, Parser, ValueEnum};
use lzw_code::{Code, CodeGenerator};
use std::env;
use std::fs::File;
use std::io::{Read, Write};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;
mod alphabets;
mod lzw_code;
mod lzw_dict;
use lzw_dict::{LzwDict, Token};
use mutable_trie::{self, TrieKey};
use mutable_trie::{Trie, TrieNode};

#[derive(Debug, Copy, Clone)]
pub struct LzwSpec {
    alphabet: ArgAlphabet,
    variable_width: bool,
    width: u8,
    min_width: u8,
    max_width: u8,
    end_code: bool,
    clear_code: bool,
    pack_msb_first: bool,
    early_change: bool,
}

// TODO: do fancier exclusive fields? min and max code width only needed for variable width.
#[derive(Parser)]
#[command(author, version, about, long_about=None)]
// #[command(propagate_version = true)]
struct LzwArgs {
    #[arg(short, long)]
    end_code: bool,

    #[arg(short, long)]
    clear_code: bool,

    #[arg(short, long)]
    variable_width: bool,

    #[arg(short, long)]
    pack_msb_first: bool,

    #[arg(long)]
    early_change: bool,

    #[arg(default_value = "encoded.txt")]
    filename: String,

    #[arg(value_enum, default_value_t=ArgAlphabet::Ascii)]
    alphabet: ArgAlphabet,

    #[arg(default_value_t = 12)]
    width: u8,

    #[arg(default_value_t = 8)]
    min_width: u8,

    #[arg(default_value_t = 16)] // requires variable-width true
    max_width: u8,
}

#[derive(Debug, Copy, Clone, ValueEnum)]
pub enum ArgAlphabet {
    _Test,
    Ascii,
    // TODO add more
}

fn main() {
    // env::set_var("RUST_BACKTRACE", "1");
    let subscriber: FmtSubscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let args = LzwArgs::parse();
    let spec = LzwSpec {
        alphabet: args.alphabet,
        variable_width: args.variable_width,
        width: args.width,
        min_width: args.min_width,
        max_width: args.max_width,
        end_code: args.end_code,
        clear_code: args.clear_code,
        pack_msb_first: args.pack_msb_first,
        early_change: args.early_change,
    };

    let mut input: Vec<Token<char>> = "tobeornottobeortobeornot"
        .chars()
        .map(Token::Value)
        .collect();
    input.push(Token::End);

    compress(spec, input).unwrap();
}

#[derive(Debug)]
enum LzwError {
    Compress(String),
    Alphabet(String),
    Trie(String),
}

// TODO: smoother way to do it with From::from etc?
impl LzwError {
    fn from_alphabet(err: alphabets::AlphabetError) -> LzwError {
        match err {
            alphabets::AlphabetError::Generate(s) => {
                LzwError::Alphabet(String::from("Generation error") + &s)
            }
        }
    }

    fn from_trie(err: mutable_trie::TrieError) -> LzwError {
        match err {
            mutable_trie::TrieError::Insert(s) => {
                LzwError::Trie(String::from("Trie Insert Error") + &s)
            }
            mutable_trie::TrieError::Search(s) => {
                LzwError::Trie(String::from("Trie Search Error") + &s)
            }
            mutable_trie::TrieError::Lzw(s) => {
                LzwError::Trie(String::from("LzwTrie LzwInsert Error") + &s)
            }
        }
    }
}

// https://planetcalc.com/9069/

// TODO: type alias TrieNode for lzwdict

fn compress<T: TrieKey + Alphabetable<T>>(
    spec: LzwSpec,
    file_vec: Vec<Token<T>>,
) -> Result<(), LzwError> {
    let mut code_gen = lzw_code::CodeGenerator::new(spec);
    let alphabet = T::generate().map_err(LzwError::from_alphabet)?;
    let initial_entries = create_initial_entries(spec, alphabet, &mut code_gen);

    let mut lzw_trie: TrieNode<Token<T>, Code> = TrieNode::new(None, None);
    lzw_trie
        .populate_initial(initial_entries)
        .map_err(LzwError::from_trie)?;

    let mut peek_file = file_vec.into_iter().peekable();

    // While not the end token, emit codes
    // Also need to cope with clear token - so probably a match
    // TODO Is there a nice way to use the peekable trait to do this loop
    // Maybe we don't actually need to match on contents- only none and some
    // Seems sub optimal to have to peek at all the tokens.
    while let Some(code_to_emit) = lzw_trie
        .lzw_insert(&mut peek_file, &mut code_gen)
        .map_err(LzwError::from_trie)?
    {
        tracing::info!("End Code emitted: {:?}", code_to_emit);
    }

    Ok(())
}

fn decompress<T>(spec: LzwSpec, code_vec: Vec<Code>) {
    // Generate initial dictionary based on T
}

fn create_initial_entries<'a, T: TrieKey + 'a>(
    spec: LzwSpec,
    mut tokens: Vec<Token<T>>,
    code_gen: &'a mut CodeGenerator,
) -> impl Iterator<Item = (Token<T>, Code)> + 'a {
    // TODO: add clear and end codes if in spec
    if spec.clear_code {
        tokens.push(Token::Clear);
    }
    if spec.end_code {
        tokens.push(Token::End);
    }
    tokens.into_iter().zip(code_gen)
}

fn b64_decode_from_file(filename: &str) -> std::io::Result<()> {
    // File is in b64 encoding
    let mut f = File::open(filename)?;
    let mut decoder = base64::read::DecoderReader::new(&mut f, &general_purpose::STANDARD);

    let mut result: Vec<u8> = Vec::new();
    decoder.read_to_end(&mut result).unwrap();
    let s_result = String::from_utf8(result).expect("Found invalid UTF-8");
    println!("{}", s_result);
    Ok(())
}

fn b64_encode_to_file(filename: &str) -> std::io::Result<()> {
    let s = b"thetest";
    let mut f: File = File::create(filename)?;
    let mut encoder = base64::write::EncoderWriter::new(&mut f, &general_purpose::STANDARD);
    encoder.write_all(s)?;
    Ok(())
}
