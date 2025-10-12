#[allow(unused)]

mod bencode;
mod bdecode;
use bdecode::{bdecode_element};

fn main() {
    let sample_file = "d8:announce14:http://tracker4:infod6:lengthi12345e4:name10:myfile.txtee".to_string();

    match bdecode_element(&sample_file.into_bytes()){
        Ok(v) => println!("{:?}", v),
        Err(e) => println!("Error: {:?}", e),
    }
}              