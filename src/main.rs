#![allow(unused)]

use std::collections::BTreeMap;
use std::fmt;

use std::env;
use std::fs;

mod bencode;
mod bdecode;
mod tracker_request;

use bencode::{BencodeValue, bencode_element};
use bdecode::bdecode_element;

#[derive(Debug, Clone)]
struct PeerInfo{
    ip: String,
    peer_id: String,
    port: u128 // to support ipv6 in the future.
}

// impl fmt::Display for PeerInfo {
    // fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // write!(f, "{:?}", self)
    // }
// }

// impl std::error::Error for PeerInfo {}


fn get_info_from_peer_dict(peer: &BencodeValue) -> Option<PeerInfo> {
    let dict = bencode::get_dictionary(peer).ok()?; // TODO: check if this is correct.

    match dict.get(&b"ip"[..]) {
        None => None,
        Some(ip) => {
            match dict.get(&b"peer_id"[..]) {
                None => None,
                Some(peer_id) => {
                    match dict.get(&b"port"[..]) {
                        None => None,
                        Some(port) => {
                            Some(PeerInfo {
                                ip: bencode::get_utf8_lossy(&ip).ok()?, 
                                peer_id: bencode::get_utf8_lossy(&peer_id).ok()?,
                                port: bencode::get_integer(&port).ok()?.try_into().unwrap() // this absolute garbage. TODO: Change return type from Option to Result. Change field type of bencoding integer from i64 to i128.
                            })
                        }
                    }
                }
            }
        }
    }
}

fn get_peer_info(tracker_response: &BTreeMap<Vec<u8>, BencodeValue>) -> Result<Option<PeerInfo>, Box<dyn std::error::Error>> { 
    match tracker_response.get(&b"peers"[..]){
        Some(val) => {
            let BencodeValue::List(peers) = val else {
                eprintln!("ERROR: 'peers' field is not a List of Dictionairies"); 
                return Err(format!("tracker_response = {tracker_response:#?}").into());
            };

            for peer in peers {
                match get_info_from_peer_dict(peer) {
                    None => {continue;},
                    Some(peer_info) => {return Ok(Some(peer_info));}
                }
            }
            Ok(None)
        },
        None => {
            eprintln!("ERROR: Tracker response does not have a field named 'peers'.");
            Err(format!("tracker_response = {tracker_response:#?}").into())
        }
    }
}

fn main() ->  Result<(), Box<dyn std::error::Error>> {
    // Get command-line arguments
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <path to .torrent file>", args[0]);
        std::process::exit(1);
    }

    // Get file path of the .torrent file.
    let path = &args[1];

    // Read file
    let bytes = fs::read(path).expect("Failed to read file");

    // Extract the structured metadata from the torrent file.
    let BencodeValue::Dictionary(torrent) = bdecode_element(&bytes)? else {
        eprintln!("ERROR: Torrent file is not a dictionary.");
        std::process::exit(1);
    };

    // Get a valid response from the tracker. Contains a list of IP addresses of the peers.
    let BencodeValue::Dictionary(tracker_response) = tracker_request::get_tracker_response(&torrent)? else {
        eprintln!("ERROR: Tracker response is not a dictionary.");
        std::process::exit(1);
    };

    let peer_info = get_peer_info(&tracker_response); 
    println!("peer info = {peer_info:#?}");

    // Communication with peer

    Ok(())
}