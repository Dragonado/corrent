#[allow(unused)]

mod bencode;
mod bdecode;
use std::fs;
use std::io;

fn main() -> io::Result<()> {
    // Path to your .torrent file
    let path = "sample.torrent";

    // Read the entire file as bytes
    let bytes = fs::read(path)?;

    // Now decode it
    match bdecode::bdecode_element(&bytes) {
        Ok(v) => println!("Decoded value = {:?}", v),
        Err(e) => println!("Error while decoding: {:?}", e),
    }

    Ok(())
}
