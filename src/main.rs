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

use std::net::TcpStream;
use std::io::prelude::*;
use std::time::Duration;
use std::io::{self, Write};

fn is_tcp_port_open(address: &str, port: u16, timeout: Duration) -> bool {
    let full_address = format!("{}:{}", address, port);
    match TcpStream::connect_timeout(&full_address.parse().unwrap(), timeout) {
        Ok(_) => true,
        Err(_) => false,
    }
}

#[derive(Debug, Default, Clone)]
struct PeerInfo{
    ip: String,
    peer_id: Vec<u8>, // 20 byte peer id.
    port: u16 // to support ipv6 in the future.
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
            match dict.get(&b"peer id"[..]) {
                None => None,
                Some(peer_id) => {
                    match dict.get(&b"port"[..]) {
                        None => None,
                        Some(port) => {
                            Some(PeerInfo {
                                ip: bencode::get_utf8_lossy(&ip).ok()?, 
                                peer_id: bencode::get_bytestring(&peer_id).ok()?,
                                port: bencode::get_integer(&port).ok()?.try_into().unwrap() // this absolute garbage. TODO: Change return type from Option to Result. Change field type of bencoding integer from i64 to i128.
                            })
                        }
                    }
                }
            }
        }
    }
}

fn get_all_peers_info(tracker_response: &BTreeMap<Vec<u8>, BencodeValue>) -> Result<Vec<PeerInfo>, Box<dyn std::error::Error>> { 
    match tracker_response.get(&b"peers"[..]){
        Some(val) => {
            let BencodeValue::List(peers) = val else {
                eprintln!("ERROR: 'peers' field is not a List of Dictionairies"); 
                return Err(format!("tracker_response = {tracker_response:#?}").into());
            };

            let mut vec = Vec::<PeerInfo>::new();
            for peer in peers {
                match get_info_from_peer_dict(peer) {
                    None => {continue;},
                    Some(peer_info) => {vec.push(peer_info);}
                }
            }
            Ok(vec)
        },
        None => {
            eprintln!("ERROR: Tracker response does not have a field named 'peers'.");
            Err(format!("tracker_response = {tracker_response:#?}").into())
        }
    }
}

fn perform_handshake(active_peer: &PeerInfo, torrent: &BTreeMap<Vec<u8>, BencodeValue>) -> Result<(), Box<dyn std::error::Error>> {
    let peer_url = active_peer.ip.clone() + ":" + &active_peer.port.to_string(); 

    let mut stream = TcpStream::connect_timeout(&peer_url.parse().unwrap(), Duration::new(60, 0))?;
    
    let pstr = b"BitTorrent protocol";
    let pstrlen = [pstr.len() as u8]; // single byte 19
    let reserved = [0u8; 8];

    let mut handshake = Vec::new();
    handshake.extend_from_slice(&pstrlen);
    handshake.extend_from_slice(pstr);
    handshake.extend_from_slice(&reserved);
    handshake.extend_from_slice(&tracker_request::get_info_hash(&torrent));
    handshake.extend_from_slice(&active_peer.peer_id);

    assert_eq!(handshake.len(), 68); // sanity check

    stream.write_all(&handshake)?;
    stream.flush()?;

    let mut buf = vec![0u8; 70];
    let num_bytes_read = stream.read(&mut buf)?;

    if num_bytes_read == 0 {
        return Err("No bytes read from connection".into());
    }

    println!("num_bytes_read = {num_bytes_read:#?}");
    println!("Buffer input in UTF-8 = {}", String::from_utf8_lossy(&buf));
    println!("Buffer input in raw = {:?}", buf);
    Ok(())
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

    let all_peers_info = get_all_peers_info(&tracker_response)?;
    if all_peers_info.len() == 0 {
        eprintln!("ERROR: Could not find any valid peer given by the tracker.");
        std::process::exit(1);
    };

    let mut active_peer = PeerInfo::default();
    let mut active_cnt = 0;
    for peer_info in &all_peers_info {
        match perform_handshake(&peer_info, &torrent) {
            Ok(()) => { break; },
            Err(_) => { continue; }
        }
    }
    
    Ok(())
}