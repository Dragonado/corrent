#![allow(unused)]

use std::collections::BTreeMap;

use std::env;
use std::fs;

mod bencode;
mod bdecode;
mod tracker_request;

use bencode::{BencodeValue, bencode_element};
use bdecode::bdecode_element;


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

    let BencodeValue::Dictionary(torrent) = bdecode_element(&bytes).expect("Failed to decode") else {
        eprintln!("ERROR: Torrent file is not a dictionary.");
        std::process::exit(1);
    };

    let decoded_response = tracker_request::get_tracker_response(&torrent)?;  

    // Communication with peer
    Ok(())
}