#[allow(unused)]

use std::fmt;
use std::collections::BTreeMap;

use crate::bencode::BencodeValue;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BdecodingError {
    // TODO: Figure out why we dont use the commented out errors.
    // NullRoot(String),
    // NonSingularRootItem(String),
    // InvalidType(String),
    MissingTerminator(String),
    IntegerError(String),
    ByteStringError(String),
    ListError(String),
    DictionaryError(String)
}

impl fmt::Display for BdecodingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for BdecodingError {}

fn parse_i64(e: &[u8]) -> Result<i64, BdecodingError> {
    match String::from_utf8(e.to_vec()) {
        Ok(s) => match s.parse::<i64>() {
            Ok(num) =>{
                // Fail on cases like 042.
                if num > 0 && s.chars().nth(0).unwrap() == '0'  {
                    return Err(BdecodingError::IntegerError(format!("Chaithu: Number contains leading zero. s = {}", s)))
                }
                
                // Fail on cases like -042.
                if num < 0 && s.chars().nth(0).unwrap() == '-' && s.chars().nth(1).unwrap() == '0'  {
                    return Err(BdecodingError::IntegerError(format!("Chaithu: Number contains leading zero. s = {}", s)))
                }

                // Fail on cases like -0 or 000
                if num == 0 && s != "0" {
                    return Err(BdecodingError::IntegerError(format!("Chaithu: Number is 0 but length of string is more than 1. s = {}", s)))
                } 

                Ok(num)
            },
            Err(err) => Err(BdecodingError::IntegerError(format!("Chaithu: {} denoting an integer was not actually an i64 integer. error: {}", &s, &err.to_string())))
        }
        Err(err) => Err(BdecodingError::IntegerError(format!("Chaithu: String denoting an integer was not UTF-8 valid.\n{}", &err.to_string())))
    }
}
 
fn get_corresponding_terminator(e: &[u8]) -> Result<usize, BdecodingError> {
    // Assert here because we only call this function if the starting character actually has a l or a d.
    assert!(e[0] == b'l' || e[0] == b'd');

    let mut index = 1; // first index is either l or d.
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
    if e.is_empty() {
        return Ok(0);
    }

    match e[0] {
        b'i' =>  Ok(1 + get_first_index(&e, b'e')?)
        ,
        b'l' | b'd'  => {
            Ok(get_corresponding_terminator(&e)? + 1)
        },
        _ => {
            let colon_index = get_first_index(&e, b':')?;
            let number = parse_i64(&e[0..colon_index])?;

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
    // get_first_element_len(&e) will validate the string for me.
    Ok(BencodeValue::ByteString(e[get_first_index(&e, b':')? + 1 .. get_first_element_len(&e)?].to_vec()))
}

// Parses bytestring of the form: iXXXXXe
fn bdecode_i64(e: &[u8]) -> Result<BencodeValue, BdecodingError> {
    if e[0] != b'i' {
        return Err(BdecodingError::IntegerError(format!("Expected integer {} to start with 'i'.", String::from_utf8_lossy(&e))))
    }
    if e[e.len()-1] != b'e' {
        return Err(BdecodingError::IntegerError(format!("Expected integer {} to end with 'e'.", String::from_utf8_lossy(&e))))
    }

    match parse_i64(&e[1..e.len()-1]){
        Ok(num) => Ok(BencodeValue::Integer(num)),
        Err(e) => Err(e) 
    }
}

// Parses bytestring of the form: lXXXXXe
// TODO: This is slow because of O(n^2) time I think. But the files are not that big so its fine.
// Consider llllll....i0eee....eeeeee worst case scenario.
fn bdecode_list(e: &[u8]) -> Result<BencodeValue, BdecodingError> {
    if e[0] != b'l' {
        return Err(BdecodingError::ListError(format!("Expected integer {} to start with 'l'.", String::from_utf8_lossy(&e))))
    }
    if e[e.len()-1] != b'e' {
        return Err(BdecodingError::ListError(format!("Expected integer {} to end with 'e'.", String::from_utf8_lossy(&e))))
    }

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
    if e[0] != b'd' {
        return Err(BdecodingError::ListError(format!("Expected integer {} to start with 'd'.", String::from_utf8_lossy(&e))))
    }
    if e[e.len()-1] != b'e' {
        return Err(BdecodingError::ListError(format!("Expected integer {} to end with 'e'.", String::from_utf8_lossy(&e))))
    }

    let e = &e[1..e.len()-1]; // Trim off first and last character.
    let mut ans = BTreeMap::<Vec::<u8>, BencodeValue>::new();

    let mut left_index_key = 0;
    
    while left_index_key < e.len() {
        let key_len = get_first_element_len(&e[left_index_key..])?;
        let left_index_val = left_index_key + key_len;
        let right_index_val = left_index_val + get_first_element_len(&e[left_index_val..])?;

        // Matching with string because we know that bdecode_bytestr allows return a bytestring array instead of list or i64 or dict.
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
pub fn bdecode_element(e: &[u8]) -> Result<BencodeValue, BdecodingError> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bencode::{bencode_element};

    // ------------------ INTEGER TESTS ------------------

    #[test]
    fn test_valid_integers() {
        assert_eq!(bdecode_element(b"i0e").unwrap(), BencodeValue::Integer(0));
        assert_eq!(bdecode_element(b"i42e").unwrap(), BencodeValue::Integer(42));
        assert_eq!(bdecode_element(b"i-5e").unwrap(), BencodeValue::Integer(-5));
    }

    #[test]
    fn test_invalid_integers() {
        assert!(bdecode_element(b"i042e").is_err());   // leading zero
        assert!(bdecode_element(b"i-042e").is_err());  // negative with leading zero
        assert!(bdecode_element(b"i00e").is_err());    // double zero
        assert!(bdecode_element(b"i-0e").is_err());    // -0 invalid
        assert!(bdecode_element(b"i5").is_err());      // missing terminator
        assert!(bdecode_element(b"iabcde").is_err());  // non-numeric
    }

    // ------------------ BYTE STRING TESTS ------------------

    #[test]
    fn test_valid_bytestrings() {
        assert_eq!(
            bdecode_element(b"4:spam").unwrap(),
            BencodeValue::ByteString(b"spam".to_vec())
        );
        assert_eq!(
            bdecode_element(b"0:").unwrap(),
            BencodeValue::ByteString(Vec::new())
        );
        assert_eq!(
            bdecode_element(b"11:hello world").unwrap(),
            BencodeValue::ByteString(b"hello world".to_vec())
        );
    }

    #[test]
    fn test_invalid_bytestrings() {
        assert!(bdecode_element(b"4spa").is_err());    // missing colon
        assert!(bdecode_element(b"4:spa").is_err());   // short content
        assert!(bdecode_element(b"-1:abc").is_err());  // negative len
        assert!(bdecode_element(b"x:spam").is_err());  // invalid number prefix
    }

    // ------------------ LIST TESTS ------------------

    #[test]
    fn test_valid_lists() {
        assert_eq!(bdecode_element(b"le").unwrap(), BencodeValue::List(vec![]));
        assert_eq!(
            bdecode_element(b"li42ee").unwrap(),
            BencodeValue::List(vec![BencodeValue::Integer(42)])
        );
        assert_eq!(
            bdecode_element(b"l4:spami42ee").unwrap(),
            BencodeValue::List(vec![
                BencodeValue::ByteString(b"spam".to_vec()),
                BencodeValue::Integer(42)
            ])
        );
        assert_eq!(
            bdecode_element(b"ll4:hiieee").unwrap(),
            BencodeValue::List(vec![BencodeValue::List(vec![BencodeValue::ByteString(b"hiie".to_vec())])])
        );
    }

    #[test]
    fn test_invalid_lists() {
        assert!(bdecode_element(b"l4:spam").is_err());  // missing terminator
        assert!(bdecode_element(b"li042ee").is_err());  // invalid element
    }

    // ------------------ DICTIONARY TESTS ------------------

    #[test]
    fn test_valid_dictionaries() {
        assert_eq!(
            bdecode_element(b"de").unwrap(),
            BencodeValue::Dictionary(BTreeMap::new())
        );

        let mut dict = BTreeMap::new();
        dict.insert(b"cow".to_vec(), BencodeValue::ByteString(b"moo".to_vec()));
        dict.insert(b"spam".to_vec(), BencodeValue::ByteString(b"eggs".to_vec()));
        assert_eq!(
            bdecode_element(b"d3:cow3:moo4:spam4:eggse").unwrap(),
            BencodeValue::Dictionary(dict)
        );
    }
    #[test]
    fn test_non_utf8_length_prefix_fails() {
        use super::*;

        let encoded = b"\xFF:abc";
        let res = bdecode_element(encoded);

        // Current buggy behavior: we expect an error about UTF-8 / Integer parsing
        assert!(res.is_err());

        match res {
            Err(BdecodingError::IntegerError(msg)) => {
                // message may vary; check it mentions UTF-8 or integer parsing
                assert!(
                    msg.contains("UTF-8") || msg.contains("integer"),
                    "unexpected error message: {}",
                    msg
                );
            }
            other => panic!("expected IntegerError but got: {:?}", other),
        }
    }  
    fn test_invalid_dictionaries() {
        assert!(bdecode_element(b"di42e4:spame").is_err()); // non-string key
        assert!(bdecode_element(b"d4:spam4:eggs").is_err()); // missing e
        assert!(bdecode_element(b"d4:zoo4:eggs3:foo4:barre").is_err()); // unsorted keys
    }

    // ------------------ MIXED / EDGE CASES ------------------

    #[test]
    fn test_empty_input() {
        assert_eq!(
            bdecode_element(b"").unwrap(),
            BencodeValue::ByteString(Vec::new())
        );
    }

    #[test]
    fn test_invalid_concatenated_items() {
        assert!(bdecode_element(b"i5ei9e").is_err()); // invalid concatenation
    }

    #[test]
    fn test_nested_structures() {
        let val = BencodeValue::Dictionary(BTreeMap::from([
            (b"list".to_vec(),
                BencodeValue::List(vec![
                    BencodeValue::ByteString(b"spam".to_vec()),
                    BencodeValue::ByteString(b"eggs".to_vec())
                ])
            )
        ]));

        assert_eq!(
            bdecode_element(b"d4:listl4:spam4:eggsee").unwrap(),
            val
        );
    }

    // ------------------ ROUND-TRIP ENCODE/DECODE ------------------

    #[test]
    fn test_round_trip() {
        let original = BencodeValue::List(vec![
            BencodeValue::Integer(42),
            BencodeValue::ByteString(b"spam".to_vec()),
        ]);
        let encoded = bencode_element(&original);
        let decoded = bdecode_element(&encoded).unwrap();
        assert_eq!(decoded, original);
    }
}
