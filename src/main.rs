#![allow(unused)]

use std::collections::BTreeMap;

use std::env;
use std::fs;
use reqwest;
use bencoding;

mod bencode;
mod bdecode;

use bencode::{BencodeValue, bencode_element};
use bdecode::bdecode_element;

use hex_literal::hex;
use sha1::{Sha1, Digest};

fn get_info(torrent: &BTreeMap<Vec<u8>, BencodeValue>) -> &BencodeValue {
    match torrent.get(&b"info"[..]){
        Some(info) => info,
        None => {
            eprintln!("ERROR: Torrent file does not contain 'info' field.");
            std::process::exit(1);
        }
    } 
}

fn get_announce_url(torrent: &BTreeMap<Vec<u8>, BencodeValue>) -> String {
    match torrent.get(&b"announce"[..]){
        Some(announce) => {
            let BencodeValue::ByteString(announce_bytes) = announce else {
                eprintln!("ERROR: 'announce' field is not a bytestring.");
                std::process::exit(1);
            };
            String::from_utf8(announce_bytes.to_vec()).expect("ERROR: 'announce' field is not UTF-8 valid.")
        }
        None => {
            eprintln!("ERROR: Torrent file does not contain 'announce' field.");
            std::process::exit(1);
        } 
    } 
}

fn escape_hash_to_string(hash: &Vec<u8>) -> String {
    hash.iter()
        .map(|b| format!("%{:02X}", b))
        .collect::<String>()
}

fn get_hash(bytes: &Vec<u8>) -> Vec<u8> {
    // Create SHA-1 hasher object.
    let mut hasher = Sha1::new();

    // process input message
    hasher.update(&bytes);

    // acquire hash digest in the form of GenericArray and then convert to Vec,
    hasher.finalize().to_vec()
}

fn get_info_hash(torrent: &BTreeMap<Vec<u8>, BencodeValue>) -> Vec<u8> {
    let info = get_info(&torrent);
    get_hash(&bencode_element(&info))
} 

fn get_random_20byte_hash() -> Vec<u8> {
    get_hash(&b"Dragonado is the goat".to_vec())
}

fn get_tracker_request_url(torrent: &BTreeMap<Vec<u8>, BencodeValue>) -> String {
    let mut url = get_announce_url(&torrent);

    url = url + "?info_hash=" + &escape_hash_to_string(&get_info_hash(&torrent)); 
    url = url + "&peer_id=" + &escape_hash_to_string(&get_random_20byte_hash());
    url = url + "&port=6881";
    url
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

    let BencodeValue::Dictionary(torrent) = bdecode_element(&bytes).expect("Failed to decode") else {
        eprintln!("ERROR: Torrent file is not a dictionary.");
        std::process::exit(1);
    };

    let get_tracker_request_url = get_tracker_request_url(&torrent);
    println!("get_tracker_request_url = {get_tracker_request_url:#?}");    
    let response = reqwest::blocking::get(get_tracker_request_url)?.bytes()?;
    println!("response = {response:#?}");    
    let decoded_response = bdecode_element(&response); 
    println!("decoded_response = {decoded_response:#?}");    

    Ok(())
}
