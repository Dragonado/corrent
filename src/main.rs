#![allow(unused)]

use std::collections::BTreeMap;

mod bencode;
use bencode::{BencodeValue, bencode_element};

#[derive(Debug, Clone, PartialEq, Eq)]
enum BdecodingError {
    NullRoot(String),
    NonSingularRootItem(String),
    InvalidType(String),
    MissingTerminator(String),
    IntegerError(String),
    ByteStringError(String),
    DictionaryError(String)
}

fn parse_i64(e: &[u8]) -> Result<BencodeValue, BdecodingError> {
    match String::from_utf8(e.to_vec()) {
        Ok(s) => match(s.parse::<i64>()) {
            Ok(num) =>{
                // Fail on cases like -042.
                if num < 0 && s.chars().nth(0).unwrap() == '0'  {
                    return Err(BdecodingError::IntegerError(format!("Chaithu: Number contains leading zero. s = {}", s)))
                }
                
                // Fail on cases like 01245.
                if num > 0 && s.chars().nth(0).unwrap() == '-' && s.chars().nth(1).unwrap() == '0'  {
                    return Err(BdecodingError::IntegerError(format!("Chaithu: Number contains leading zero. s = {}", s)))
                }

                // Fail on cases like -0 or 000
                if num == 0 && s != "0" {
                    return Err(BdecodingError::IntegerError(format!("Chaithu: Number is 0 but length of string is more than 1. s = {}", s)))
                } 

                Ok(BencodeValue::Integer(num))
            },
            Err(err) => Err(BdecodingError::IntegerError(format!("Chaithu: String denoting an integer was not actually an i64 integer\n{}", &err.to_string())))
        }
        Err(err) => Err(BdecodingError::IntegerError(format!("Chaithu: String denoting an integer was not UTF-8 valid.\n{}", &err.to_string())))
    }
}
 
fn get_corresponding_terminator(e: &[u8]) -> Result<usize, BdecodingError> {
    println!("get_corresponding_terminator: e = {}", String::from_utf8_lossy(&e));

    assert!(e[0] == b'i' || e[0] == b'l' || e[0] == b'd');

    let mut opening_cnt = 0i64;

    let mut index = 1;
    while index < e.len() {
        match e[index] {
            b'e' => {return Ok(index);},
            _ => {index += get_first_element_len(&e[index..])?;}
        }
    }

    return Err(BdecodingError::MissingTerminator(format!("Chaithu: Could not find corresponding terminating character 'e' for {}.", String::from_utf8_lossy(&e))));
}


fn get_first_index(e: &[u8], ch: u8) -> Result<usize, BdecodingError> {
    match e.into_iter().position(|x| *x == ch) {
        Some(i) => Ok(i),
        None => Err(BdecodingError::MissingTerminator(format!("Chaithu: Expected {} to end with {} but didn't,", String::from_utf8_lossy(&e), String::from_utf8_lossy(&vec![ch]))))
    }
}
// If string consists of [1][2]..[n] elements. This function returns the number of bytes that stores the 1st element.
// In particular if only one element exists then it returns the size of the vector.
fn get_first_element_len(e: &[u8]) -> Result<usize, BdecodingError> {
    println!("get_first_element_len: e = {}", String::from_utf8_lossy(&e));

    if e.is_empty() {
        return Ok(0);
    }

    match e[0] {
        b'i' =>  Ok(get_first_index(&e, b'e')?)
        ,
        b'l' | b'd'  => {
            Ok(get_corresponding_terminator(&e)? + 1)
        },
        _ => {
            let colon_index = get_first_index(&e, b':')?;
            let number = match parse_i64(&e[0..colon_index]) {
                Ok(BencodeValue::Integer(n)) => n,
                _ => return Err(BdecodingError::ByteStringError(format!("Chaithu: Invalid length prefix in {}", String::from_utf8_lossy(&e)))),
            };

            if number < 0 {
                return Err(BdecodingError::ByteStringError(format!("Chaithu: Negative length string found in {}.", String::from_utf8_lossy(&e))));
            }

            let len = number as usize;
            let start_of_str = colon_index + 1;
            let end_of_str = start_of_str + len;

            if end_of_str > e.len() {
                return Err(BdecodingError::ByteStringError(format!("Chaithu: Length described exceeds byte string length in {}.", String::from_utf8_lossy(&e))));
            }
            
            Ok(end_of_str)
        }        
    }
}

// Parses byestring of the form: num:XXXXXX
fn bdecode_bytestr(e: &[u8]) -> Result<BencodeValue, BdecodingError> {
    println!("bdecode_bytestr: e = {}", String::from_utf8_lossy(&e));

    assert!(e[0] != b'i' && e[0] != b'l' && e[0] != b'd');
    Ok(BencodeValue::ByteString(e[get_first_index(&e, b':')? + 1 .. get_first_element_len(&e)?].to_vec()))
}

// Parses bytestring of the form: iXXXXXe
fn bdecode_i64(e: &[u8]) -> Result<BencodeValue, BdecodingError> {
    println!("bencode_i64: e = {}", String::from_utf8_lossy(&e));

    assert!(e[0] == b'i');
    assert!(e[e.len()-1] == b'e');

    parse_i64(&e[1..e.len()-1])
}

// Parses bytestring of the form: lXXXXXe
// TODO: This is slow because of O(n^2) time I think. But the files are not that big so its fine.
// Consider llllll....i0eee....eeeeee worst case scenario.
fn bdecode_list(e: &[u8]) -> Result<BencodeValue, BdecodingError> {
    println!("bencode_list: e = {}", String::from_utf8_lossy(&e));

    assert!(e[0] == b'l');
    assert!(e[e.len()-1] == b'e');
    let e = &e[1..e.len()-1]; // trim the first and last character.
    let mut left_index = 0;
    let mut ans = Vec::<BencodeValue>::new();

    while left_index < e.len() {
        let right_index = left_index + get_first_element_len(&e[left_index..])?;
        ans.push(bdecode_element(&e[left_index..right_index])?);
        left_index = right_index;
    }

    Ok(BencodeValue::List(ans))
}

// Parses bytestring of the form: dXXXXXe
// TODO: This is slow because of O(n^2) time I think. But the files are not that big so its fine.
// Consider ddddddd....4:abcdi0eee....eeeeee worst case scenario.
fn bdecode_dicitionary(e: &[u8]) -> Result<BencodeValue, BdecodingError> {
    println!("bencode_dictionary: e = {}", String::from_utf8_lossy(&e));
    assert!(e[0] == b'd');
    assert!(e[e.len()-1] == b'e');
    let e = &e[1..e.len()-1]; // Trim off first and last character.
    let mut ans = BTreeMap::<Vec::<u8>, BencodeValue>::new();

    let mut left_index_key = 0;
    
    while left_index_key < e.len() {
        let key_len = get_first_element_len(&e[left_index_key..])?;
        let left_index_val = left_index_key + key_len;
        let right_index_val = left_index_val + get_first_element_len(&e[left_index_val..])?;

        println!("left_index_key = {}, left_index_val = {}, right_index_val = {}", left_index_key, left_index_val, right_index_val); 

        let BencodeValue::ByteString(key) = bdecode_bytestr(&e[left_index_key..left_index_val])? else {unreachable!()};
        let val = bdecode_element(&e[left_index_val..right_index_val])?;

        if let Some(latest_keyval) = ans.last_entry() {
            if *latest_keyval.key() >= key {
                return Err(BdecodingError::DictionaryError(format!("Chaithu: Expected keys to be in strictly increasing lexicographical order.\n{}", String::from_utf8_lossy(&e))));
            }
        }
        ans.insert(key, val);
        left_index_key = right_index_val;
    }

    Ok(BencodeValue::Dictionary(ans))
}

// parses the entire string as one entity. So you can't have something like i5ei9e to denote 5,9. You have to wrap it in a list.
fn bdecode_element(e: &[u8]) -> Result<BencodeValue, BdecodingError> {
    println!("bencode_element: e = {}", String::from_utf8_lossy(&e));

    if e.is_empty() {
        return Ok(BencodeValue::ByteString(Vec::new()));
    }

    match e[0] {
        b'i' | b'l' | b'd'  => {
            if e.last() != Some(&b'e') {
                return Err(BdecodingError::MissingTerminator(format!("Chaithu: Expected {} to end with 'e' but didn't.\n", String::from_utf8_lossy(&e))));
            }

            match e[0] {
                b'i' => bdecode_i64(&e),
                b'l' => bdecode_list(&e),
                b'd' => bdecode_dicitionary(&e),
                _ => unreachable!()
            }
        },
        _ => bdecode_bytestr(&e),
    }
}

fn main() {
    let int = BencodeValue::Integer(41);
    let bytestr = BencodeValue::ByteString(b"hello".to_vec());
    let list = BencodeValue::List(vec![int.clone(), bytestr.clone()]);
    let dict = BencodeValue::Dictionary(BTreeMap::from([(b"hello".to_vec(), list.clone())]));

    let val = dict; 

    let e1 = bencode_element(&val);
    let d1 = bdecode_element(&e1);

    assert_eq!(d1, Ok(val));

}
