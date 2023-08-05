use base64::engine::general_purpose;
use clap::{ArgAction, Args, Parser, ValueEnum};
use std::env;
use std::fs::File;
use std::io::{Read, Write};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;
mod alphabets;
mod lzw_code;
mod lzw_token;
mod trie_dictionary;
use trie_dictionary::TrieDictionary;

#[derive(Debug, Copy, Clone)]
pub struct LzwSpec {
    alphabet: alphabets::Alphabet,
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
// TODO: Have a second option to point to a
#[derive(Parser)]
#[command(author, version, about, long_about=None)]
// #[command(propagate_version = true)]
struct LzwArgs {
    #[arg(short, long)]
    end_code: bool,

    #[arg(short, long)]
    clear_code: bool,

    #[arg(short, long, action=ArgAction::SetFalse)]
    variable_width: bool,

    #[arg(short, long)] // TODO: work out what correct default is
    pack_msb_first: bool,

    #[arg(long, action=ArgAction::SetFalse)]
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
        alphabet: alphabets::Alphabet::new(args.alphabet),
        variable_width: args.variable_width,
        width: args.width,
        min_width: args.min_width,
        max_width: args.max_width,
        end_code: args.end_code,
        clear_code: args.clear_code,
        pack_msb_first: args.pack_msb_first,
        early_change: args.early_change,
    };
    compress(spec);
    // b64_encode_to_file(&args.filename).unwrap();
    // b64_decode_from_file(&args.filename).unwrap();
}

// https://planetcalc.com/9069/

fn compress(spec: LzwSpec) {
    tracing::debug!("Starting compression");
    let mut code_gen = lzw_code::CodeGenerator::new(spec);
    let alphabet = alphabets::generate_ascii(); //TODO generate from the spec
    let dict = TrieDictionary::new(spec, &mut code_gen, alphabet);

    //let test_source: Vec<char> = "tobeornottobetobeornottobe".chars().collect();
    // Repeatedly read, evaluate from input, Emit to output
}

fn decompress() {
    tracing::debug!("Starting decompression")
}

fn b64_decode_from_file(filename: &str) -> std::io::Result<()> {
    // File is in b64 encoding
    tracing::debug!("Decoding {} from b64 to u8 characters", filename);
    let mut f = File::open(filename)?;
    let mut decoder = base64::read::DecoderReader::new(&mut f, &general_purpose::STANDARD);

    let mut result: Vec<u8> = Vec::new();
    decoder.read_to_end(&mut result).unwrap();
    let s_result = String::from_utf8(result).expect("Found invalid UTF-8");
    println!("{}", s_result);
    Ok(())
}

fn b64_encode_to_file(filename: &str) -> std::io::Result<()> {
    tracing::debug!("Encoding from u8 characters to b64 in {}", filename);
    let s = b"thetest";
    let mut f: File = File::create(filename)?;
    let mut encoder = base64::write::EncoderWriter::new(&mut f, &general_purpose::STANDARD);
    encoder.write_all(s)?;
    Ok(())
}
