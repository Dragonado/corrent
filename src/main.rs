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

fn bdecode_i64(e: &[u8]) -> Result<BencodeValue, BdecodingError>{
    match String::from_utf8(e.to_vec()) {
        Ok(s) => match(s.parse::<i64>()) {
            Ok(num) => Ok(BencodeValue::Integer(num)),
            Err(err) => Err(BdecodingError::IntegerError("Chaithu: String denoting an integer was not actually an i64 integer\n".to_owned() + &err.to_string()))
        }
        Err(err) => Err(BdecodingError::IntegerError("Chaithu: String denoting an integer was not UTF-8 valid.\n".to_owned() + &err.to_string()))
    }
}

fn bdecode_bytestr(e: &[u8]) -> BencodeValue {
    BencodeValue::ByteString(e.to_vec())
}

fn bdecode_list(e: &[u8]) -> Result<BencodeValue, BdecodingError> {
    todo!()
}

fn bdecode_dicitionary(e: &[u8]) -> Result<BencodeValue, BdecodingError> {
    todo!()
}  

fn bdecode_element(e: &[u8]) -> Result<BencodeValue, BdecodingError> {
    if e.is_empty() {
        return Ok(BencodeValue::ByteString(b"".to_vec()));
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

fn main() {
    let val = BencodeValue::Integer(41);
    let val = BencodeValue::ByteString(b"hello".to_vec());

    let e1 = bencode_element(&val);
    let d1 = bdecode_element(&e1);

    assert_eq!(d1, Ok(val));

}
