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

fn bdecode_i64(e: &Vec<u8>) -> Result<BencodeValue::Integer, BdecodingError::IntegerError>{
    match String::from_utf8(e) {
        Ok(s) => match(s.parse::<i64>()) {
            Ok(num) => BencodeValue::Integer(num),
            Err(s) => BdecodingError::IntegerError("Chaithu: String denoting an integer was not actually an i64 integer\n" + s)
        }
        Err(s) => BdecodingError::IntegerError("Chaithu: String denoting an integer was not UTF-8 valid.\n" + s)
    }
}

fn bdecode_element(e: &Vec<u8>) -> Result<BencodeValue, BdecodingError> {
    if e.empty() {
        BencodeValue::ByteString(b"".to_vec())
    }

    
    // i XXXX e is an integer
    // l XXXX e is a list
    // d XXXX e is a dictionary
    // num: XXXXX is a bytestring.
    Ok(BencodeValue::Integer(42))
}

fn main() {
    let val = BencodeValue::Integer(41);
    let e1 = bencode_element(&val);
    let d1 = bdecode_element(&e1);

    assert_eq!(d1, Ok(val));

}
