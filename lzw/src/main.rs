use base64::engine::general_purpose;
use clap::Parser;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Parser)]
#[command(author, version, about, long_about=None)]
// #[command(propagate_version = true)]
struct Args {
    #[arg(default_value = "encoded.txt")]
    filename: String,
}

// https://planetcalc.com/9069/

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

fn main() {
    let subscriber: FmtSubscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let args = Args::parse();
    b64_encode_to_file(&args.filename).unwrap();
    b64_decode_from_file(&args.filename).unwrap();
}
