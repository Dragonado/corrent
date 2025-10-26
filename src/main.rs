#[allow(unused)]

use std::collections::BTreeMap;

use std::env;
use std::fs;
use reqwest;

mod bencode;
mod bdecode;

use hex_literal::hex;
use sha1::{Sha1, Digest};

fn get_info(torrent: &bencode::BencodeValue) -> bencode::BencodeValue {
    match torrent.get(b"info") {
        Some(info) => info,
        None => panic!(),
    }
}

fn get_info_hash(torrent: &bencode::BencodeValue::Dictionary) -> Vec<u8> {
    let info = get_info(&torrent);
    // Create SHA-1 hasher object.
    let mut hasher = Sha1::new();

    // process input message
    hasher.update(info);
    let info_hash = hasher.finalize();

    // acquire hash digest in the form of GenericArray and then convert to Vec,
    hasher.finalize().to_vec()
} 

fn main() ->  Result<(), Box<dyn std::error::Error>> {
    // Get command-line arguments
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <path to .torrent file>", args[0]);
        std::process::exit(1);
    }

    let path = &args[1];

    // Read file
    let bytes = fs::read(path).expect("Failed to read file");

    let torrent = bencode::BencodeValue::Dictionary(decode::bdecode_element(&bytes).expect("Failed to decode")) else {
         eprintln!("Given torrent file is not a dictionary"); panic!(); 
    }; 

    println!("Decoded torrent = {torrent:#?}");

    // let body = reqwest::blocking::get("https://www.rust-lang.org")?
    // .text()?;

    Ok(())
}
