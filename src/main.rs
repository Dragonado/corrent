#![allow(unused)]

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

// parses bytestring of the form: iXXXXXe
fn bdecode_i64(e: &[u8]) -> Result<BencodeValue, BdecodingError> {
    assert!(e[0] == b'i');
    assert!(e.last() == b'e');

    let e = e[1..e.len()-1]; // trim the first and last character.
    match String::from_utf8(e.to_vec()) {
        Ok(s) => match(s.parse::<i64>()) {
            Ok(num) =>{

                // Fail on cases like -042.
                if num < 0 && s.chars().nth(0).unwrap() == '0'  {
                    return Err(BdecodingError::IntegerError(format!("Chaithu: Number contains leading zero. s = {}", s)))
                }
                
                // Fail on cases like 01245.
                if num > 0 && s.chars().nth(1).unwrap() == '0'  {
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

fn bdecode_bytestr(e: &[u8]) -> Result<BencodeValue, BdecodingError> {
    match e.into_iter().position(|x| *x == b':') {
        Some(colon_index) => {
            let number = match bdecode_i64(&e[0..colon_index]) {
                Ok(BencodeValue::Integer(n)) => n,
                _ => return Err(BdecodingError::ByteStringError(format!("Chaithu: Invalid length prefix in {}", String::from_utf8_lossy(&e)))),
            };

            if number < 0 {
                return Err(BdecodingError::ByteStringError(format!("Chaithu: Negative length string found in {}.", String::from_utf8_lossy(&e))));
            }

            let len = number as usize;
            let start_of_str = index + 1;
            let end_of_str = start_of_str + len;

            if end_of_str != e.len() {
                return Err(BdecodingError::ByteStringError(format!("Chaithu: Length described does not match the size of the byte string in {}.", String::from_utf8_lossy(&e))));
            }
            
            Ok(BencodeValue::ByteString(e[start_of_str..end_of_str].to_vec()))
        },
        None => Err(BdecodingError::MissingTerminator(format!("Chaithu: Expected {} to end with ':' but didn't,", String::from_utf8_lossy(&e))))
    }
}

// TODO: This is slow because of O(n^2) time I think. But the files are not that big so its fine.
// Consider llllll....i0eee....eeeeee
fn bdecode_list(e: &[u8]) -> Result<BencodeValue, BdecodingError> {
    let e = e[1..e.lend()-1]; // trim the first and last character.
    let mut left_index = 0;
    let mut ans = BencodeValue::List(Vec::new());

    while left_index < e.len() {
        let right_index = get_first_element_len(&e[left_index..e.len()]).unwrap();
        ans.push(bdecode_element(e[left_index..right_index]).unwrap());
        left_index = right_index;
    }

    ans
}

fn bdecode_dicitionary(e: &[u8]) -> Result<BencodeValue, BdecodingError> {
    let e = e[1..e.len()-1];
    let mut ans = BencodeValue::Dictionary(BTreeMap::new());


    let mut left_index_key = 0;
    
    // TODO: check for infinite loop case for "ddedee"
    while left_index_key < e.len() {
        if let bytestr_len != Some(get_first_element_len(e[left_index_key..e.len()])){
            return Err(BdecodingError::DictionaryError(format!("Chaithu: Expected key to be bytestring but found something else.\n{}", String::from_utf8(&e))));
        }
        
        let left_index_val = left_index + bytestr_len;
        let key:Vec<u8> = BencodeValue::ByteString(bdecode_bytestr(e[left_index_key..left_index_val]).unwrap());
        if let right_index_val != Some(get_first_element_len(e[left_index_val..e.len()])){
            return Err(BdecodingError::DictionaryError(format!("Chaithu: Expected key to be bytestring but found something else.\n{}", String::from_utf8(&e))));
        }

        let val = bdecode_element(e[left_index_val..right_index_val]);

        if let Some(latest_keyval) == ans.last_entry() {
            if *entry.key() >= key {
                return Err(BdecodingError::DictionaryError(format!("Chaithu: Expected keys to be in strictly increasing lexicographical order.\n{}", String::from_utf8(&e))));
            }
        }
        ans.push({key, val});
        left_index_key = right_index_val;
    }

    ans
}

fn bdecode_element(e: &[u8]) -> Result<BencodeValue, BdecodingError> {
    if e.is_empty() {
        return Ok(e);
    }

    match e[0] {
        b'i' | b'l' | b'd'  => {
            if e.last() != Some(&b'e') {
                return Err(BdecodingError::MissingTerminator(format!("Chaithu: Expected {} to end with 'e' but didn't.\n", String::from_utf8_lossy(&e))));
            }

            let inner_content = &e[1..e.len()-1];
            match e[0] {
                b'i' => bdecode_i64(inner_content),
                b'l' => bdecode_list(inner_content),
                b'd' => bdecode_dicitionary(inner_content),
                _ => unreachable!()
            }
        },
        _ => match e.into_iter().position(|x| *x == b':') {
            Some(index) => {
                let number = match bdecode_i64(&e[0..index]) {
                    Ok(BencodeValue::Integer(n)) => n,
                    _ => return Err(BdecodingError::ByteStringError(format!("Chaithu: Invalid length prefix in {}", String::from_utf8_lossy(&e)))),
                };

                if number < 0 {
                    return Err(BdecodingError::ByteStringError(format!("Chaithu: Negative length string found in {}.", String::from_utf8_lossy(&e))));
                }

                let len = number as usize;
                let start_of_str = index + 1;
                let end_of_str = start_of_str + len;

                if end_of_str != e.len() {
                    return Err(BdecodingError::ByteStringError(format!("Chaithu: Length described does not match the size of the byte string in {}.", String::from_utf8_lossy(&e))));
                }

                Ok(bdecode_bytestr(&e[start_of_str..end_of_str]))
            },
            None => Err(BdecodingError::MissingTerminator(format!("Chaithu: Expected {} to end with ':' but didn't,", String::from_utf8_lossy(&e))))
        }
    }
}

// If string consists of [1][2]..[n] elements. This function returns the number of bytes that stores the 1st element.
// In particular if only one element exists then it returns the size of the vector.
fn get_first_element_len(e: &[u8]) -> Result<usize, BdecodingError> {
    if e.is_empty() {
        return Ok(0);
    }

    match e[0] {
        b'i' | b'l' | b'd'  => {

            // TODO: this is wrong. need to maintain a stack.
            if let e_index != Some(e.iter().position(|x| *x == b'e')) {
                return Err(BdecodingError::MissingTerminator(format!("Chaithu: Expected {} to end with 'e' but didn't.\n", String::from_utf8_lossy(&e))));
            }

            Ok(e_index + 1)
        },
        _ => match e.into_iter().position(|x| *x == b':') {
            Some(colon_index) => {
                let number = match bdecode_i64(&e[0..colon_index]) {
                    Ok(BencodeValue::Integer(n)) => n,
                    _ => return Err(BdecodingError::ByteStringError(format!("Chaithu: Invalid length prefix in {}", String::from_utf8_lossy(&e)))),
                };

                if number < 0 {
                    return Err(BdecodingError::ByteStringError(format!("Chaithu: Negative length string found in {}.", String::from_utf8_lossy(&e))));
                }

                let len = number as usize;
                let start_of_str = index + 1;
                let end_of_str = start_of_str + len;

                if end_of_str > e.len() {
                    return Err(BdecodingError::ByteStringError(format!("Chaithu: Length described does not match the size of the byte string in {}.", String::from_utf8_lossy(&e))));
                }
                
                Ok(end_of_str)
            },
            None => Err(BdecodingError::MissingTerminator(format!("Chaithu: Expected {} to end with ':' but didn't,", String::from_utf8_lossy(&e))))
        }
    }
}

// Returns the decoded value of the first bencode element in the ByteString bencode.
fn bdecode_first_element(e: &[u8]) -> Result<BencodeValue, BdecodingError> {
    if e.is_empty() {
        return Ok(e);
    }

    match e[0] {
        b'i' | b'l' | b'd'  => {
            if let terminating_index != Some(e.iter().position(|x| *x == b'e')) {
                return Err(BdecodingError::MissingTerminator(format!("Chaithu: Expected {} to end with 'e' but didn't.\n", String::from_utf8_lossy(&e))));
            }

            let inner_content = &e[1..terminating_index]; // Trim the first and last character.
            match e[0] {
                b'i' => bdecode_i64(inner_content),
                b'l' => bdecode_list(inner_content),
                b'd' => bdecode_dicitionary(inner_content),
                _ => unreachable!()
            }
        },
        _ => match e.into_iter().position(|x| *x == b':') {
            Some(terminating_index) => {
                let number = match bdecode_i64(&e[0..terminating_index]) {
                    Ok(BencodeValue::Integer(n)) => n,
                    _ => return Err(BdecodingError::ByteStringError(format!("Chaithu: Invalid length prefix in {}", String::from_utf8_lossy(&e)))),
                };

                if number < 0 {
                    return Err(BdecodingError::ByteStringError(format!("Chaithu: Negative length string found in {}.", String::from_utf8_lossy(&e))));
                }

                let len = number as usize;
                let start_of_str = index + 1;
                let end_of_str = start_of_str + len;

                if end_of_str != e.len() {
                    return Err(BdecodingError::ByteStringError(format!("Chaithu: Length described does not match the size of the byte string in {}.", String::from_utf8_lossy(&e))));
                }

                Ok(bdecode_bytestr(&e[start_of_str..end_of_str]))
            },
            None => Err(BdecodingError::MissingTerminator(format!("Chaithu: Expected {} to end with ':' but didn't,", String::from_utf8_lossy(&e))))
        }
    }
}
fn main() {
    let val = BencodeValue::Integer(41);
    let val = BencodeValue::ByteString(b"hello".to_vec());

    let e1 = bencode_element(&val);
    let d1 = bdecode_element(&e1);

    assert_eq!(d1, Ok(val));

}
