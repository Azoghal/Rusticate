use base64::alphabet;
use base64::engine::general_purpose;
use clap::{Parser, ValueEnum};
use std::fs::File;
use std::io::{Read, Write};
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;
mod alphabets;
mod trie_dictionary;

#[derive(Debug)]
pub struct LzwSpec {
    alphabet: Alphabet,
    variable_width: bool,
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
struct Args {
    #[arg(default_value = "encoded.txt")]
    filename: String,

    #[arg(value_enum, default_value_t=Alphabet::Ascii)]
    alphabet: Alphabet,

    #[arg(default_value_t = false)]
    variable_width: bool,

    #[arg(default_value_t = true)]
    end_code: bool,

    #[arg(default_value_t = true)]
    clear_code: bool,

    #[arg(default_value_t = 16)] // requires variable-width true
    max_width: u8,

    #[arg(default_value_t = 9)]
    // requires variable-width true. Commonly inferred from alphabet rather than specified
    min_width: u8,

    #[arg(default_value_t = true)]
    pack_msb_first: bool,

    #[arg(default_value_t = false)]
    early_change: bool,
}

#[derive(Debug, Copy, Clone, ValueEnum)]
pub enum Alphabet {
    Ascii,
    // TODO add more
}

fn main() {
    let subscriber: FmtSubscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let args = Args::parse();
    let spec = LzwSpec {
        alphabet: args.alphabet,
        variable_width: args.variable_width,
        min_width: args.min_width,
        max_width: args.max_width,
        end_code: args.end_code,
        clear_code: args.clear_code,
        pack_msb_first: args.pack_msb_first,
        early_change: args.early_change,
    };
    compress(spec);
    b64_encode_to_file(&args.filename).unwrap();
    b64_decode_from_file(&args.filename).unwrap();
}

// https://planetcalc.com/9069/

fn compress(spec: LzwSpec) {
    // Initialize dictionary from spec
    let dict = trie_dictionary::TrieDictionary::new(spec);
    // Initialize Trie from dictionary
    // Repeatedly read, evaluate from input, Emit to output
}

fn decompress() {}

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
