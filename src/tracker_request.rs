
use hex_literal::hex;
use sha1::{Sha1, Digest};
use std::collections::BTreeMap;

use reqwest;

use crate::bencode::{BencodeValue, bencode_element};
use crate::bdecode::bdecode_element;


fn get_info(torrent: &BTreeMap<Vec<u8>, BencodeValue>) -> &BencodeValue {
    match torrent.get(&b"info"[..]){
        Some(info) => info,
        None => {
            eprintln!("ERROR: Torrent file does not contain 'info' field.");
            std::process::exit(1);
        }
    } 
}

// TODO: Add support for multi-tracker urls.
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
    // TODO: Add more fields like byes downloaded, uploaded, etc,.
    url
}

pub fn get_tracker_response(torrent: &BTreeMap<Vec<u8>, BencodeValue>) -> Result<BencodeValue, Box<dyn std::error::Error>> {
    let get_tracker_request_url = get_tracker_request_url(&torrent);
    // TODO: Handle the case when the tracker returns a compact format response.
    let response = reqwest::blocking::get(get_tracker_request_url)?.bytes()?;   
    let decoded_response = bdecode_element(&response)?;

    let BencodeValue::Dictionary(ref dict) = decoded_response else {
        eprintln!("ERROR: Tracker response is not a dictionary.");
        return Err(format!("response = {response:#?}").into()); // something;
    };
    match dict.get(&b"failure reason"[..]) {
        Some(e) => {
            eprintln!("ERROR: Tracker response returned a failure reason. {response:#?}");
            Err(format!("decoded_response = {decoded_response:#?}").into())
        } 
        None => Ok(decoded_response)
    } 
}
