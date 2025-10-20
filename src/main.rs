use std::env;
use std::fs;

mod bencode;
mod bdecode;

fn main() {
    // Get command-line arguments
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <path to .torrent file>", args[0]);
        std::process::exit(1);
    }

    let path = &args[1];

    // Read file
    let bytes = fs::read(path).expect("Failed to read file");

    let v = bdecode::bdecode_element(&bytes).expect("Failed to decode");
    println!("{:#?}", v);
}
