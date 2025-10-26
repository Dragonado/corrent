#![allow(dead_code)]
#[allow(unused)]

use std::collections::BTreeMap;
use std::fmt;
use std::fmt::Debug;

/// Public enum that callers will use
#[derive(Clone, PartialEq, Eq)]
pub enum BencodeValue {
    Integer(i64),
    ByteString(Vec<u8>),
    List(Vec<BencodeValue>),
    Dictionary(BTreeMap<Vec<u8>, BencodeValue>),
}

pub fn get_dictionary(b: &BencodeValue) -> Result<BTreeMap<Vec<u8>, BencodeValue>, Box<dyn std::error::Error>> {
    let BencodeValue::Dictionary(dict) = b else {
        eprintln!("ERROR: Expected bencode value to be a dicitionary");
        return Err(format!("bencode_value = {b:#?}").into());
    };
    return Ok(dict.clone());
}

pub fn get_list(b: &BencodeValue) -> Result<Vec<BencodeValue>, Box<dyn std::error::Error>> {
    let BencodeValue::List(vec) = b else {
        eprintln!("ERROR: Expected bencode value to be a list");
        return Err(format!("bencode_value = {b:#?}").into());
    };
    return Ok(vec.clone());
}


pub fn get_integer(b: &BencodeValue) -> Result<i64, Box<dyn std::error::Error>> {
    let BencodeValue::Integer(i) = b else {
        eprintln!("ERROR: Expected bencode value to be an integer.");
        return Err(format!("bencode_value = {b:#?}").into());
    };
    return Ok(i.clone());
}

pub fn get_bytestring(b: &BencodeValue) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let BencodeValue::ByteString(s) = b else {
        eprintln!("ERROR: Expected bencode value to be a bytestring.");
        return Err(format!("bencode_value = {b:#?}").into());
    };
    return Ok(s.clone());
}

pub fn get_utf8_lossy(b: &BencodeValue) -> Result<String, Box<dyn std::error::Error>> {
    let s = get_bytestring(b)?;
    return Ok(String::from_utf8_lossy(&s).to_string());
}
// pub fn get_utf8_lossy(b: &BencodeValue) -> 

impl Debug for BencodeValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BencodeValue::Integer(i) => write!(f, "Integer({})", i),
            BencodeValue::ByteString(bytes) => {
                // Convert bytes to UTF-8 lossy (so invalid UTF-8 doesn't panic)
                let s = String::from_utf8_lossy(bytes);
                write!(f, "ByteString(\"{}\")", s)
            }
            BencodeValue::List(list) => {
                write!(f, "List(")?;
                f.debug_list().entries(list).finish()?;
                write!(f, ")")
            }
            BencodeValue::Dictionary(map) => {
                write!(f, "Dictionary(")?;
                f.debug_map()
                    .entries(map.iter().map(|(k, v)| {
                        (String::from_utf8_lossy(k), v)
                    }))
                    .finish()?;
                write!(f, ")")
            }
        }
    }
}

/// Public entry-point encoder
pub fn bencode_element(b: &BencodeValue) -> Vec<u8> {
    match b {
        BencodeValue::Integer(i)    => bencode_i64(*i),
        BencodeValue::ByteString(s) => bencode_bytestr(s),
        BencodeValue::List(l)       => bencode_list(l),
        BencodeValue::Dictionary(d) => bencode_dict(d),
    }
}

fn bencode_i64(a: i64) -> Vec<u8> {
    ("i".to_owned() + &a.to_string() + "e").into_bytes()
}

fn bencode_bytestr(s: &Vec<u8>) -> Vec<u8> {
    let mut ans = (s.len().to_string() + ":").into_bytes();
    ans.extend(s);
    ans
}

fn bencode_list(v: &Vec<BencodeValue>) -> Vec<u8> {
    let mut ans = b"l".to_vec();
    for e in v {
        ans.append(&mut bencode_element(&e));
    }
    ans.push(b'e');
    ans
}

fn bencode_dict(d: &BTreeMap<Vec<u8>, BencodeValue>) -> Vec<u8> {
    let mut ans = b"d".to_vec();
    for (key, val) in d {
        ans.append(&mut bencode_bytestr(&key));
        ans.append(&mut bencode_element(&val));
    }
    ans.push(b'e');
    ans
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_integer() {
        assert_eq!(
            bencode_element(&BencodeValue::Integer(42)),
            b"i42e".to_vec()
        );
    }

    #[test]
    fn encode_byte_string() {
        assert_eq!(
            bencode_element(&BencodeValue::ByteString(b"spam".to_vec())),
            b"4:spam".to_vec()
        );
    }

    #[test]
    fn encode_list() {
        assert_eq!(
            bencode_element(&BencodeValue::List(vec![
                BencodeValue::ByteString(b"spam".to_vec()),
                BencodeValue::Integer(42),
            ])),
            b"l4:spami42ee".to_vec()
        );
    }
}