pub mod btype;

pub use btype::{BDict, BInt, BList, BString, BType};

use std::collections::HashMap;
use std::str;

use crate::utility::to_vec;

pub fn decode(input: &Vec<u8>) -> (Box<dyn BType>, u32) {
    let cur = input[0];
    if cur.is_ascii_digit() {
        let (result, offset) = decode_string(input);
        return (Box::new(result), offset);
    } else {
        match cur {
            b'd' => {
                let (result, offset) = decode_dict(input);
                return (Box::new(result), offset);
            }
            b'l' => {
                let (result, offset) = decode_list(input);
                return (Box::new(result), offset);
            }
            b'i' => {
                let (result, offset) = decode_int(input);
                return (Box::new(result), offset);
            }
            _ => panic!("Not a valid type"),
        }
    }
}

fn decode_string(input: &Vec<u8>) -> (BString, u32) {
    let mut delimiter_pos = input.iter().position(|x| x == &b':').unwrap();
    let length: usize = str::from_utf8(&input[0..delimiter_pos])
        .unwrap()
        .parse()
        .unwrap();
    delimiter_pos += 1;
    let result = input.get(delimiter_pos..delimiter_pos + length).unwrap();
    (
        BString::new(&to_vec(result)),
        (delimiter_pos + length) as u32,
    )
}

fn decode_int(input: &Vec<u8>) -> (BInt, u32) {
    let delimiter_pos = input.iter().position(|x| x == &b'e').unwrap();

    let result = str::from_utf8(&input[1..delimiter_pos])
        .unwrap()
        .parse()
        .unwrap();
    (BInt::new(result), (delimiter_pos + 1) as u32)
}

fn decode_list(input: &Vec<u8>) -> (BList, u32) {
    let mut starting_pos: usize = 1;
    let mut decoded: BList = BList::new(Vec::new());

    loop {
        if &input[starting_pos..starting_pos + 1] == b"e" {
            starting_pos += 1;
            break;
        }

        let (result, offset) = decode(&to_vec(&input[starting_pos..]));
        decoded.push(result);
        starting_pos += offset as usize;
    }

    (decoded, (starting_pos) as u32)
}

fn decode_dict(input: &Vec<u8>) -> (BDict, u32) {
    let mut starting_pos: usize = 1;
    let mut decoded: BDict = BDict::new(HashMap::new());

    loop {
        if &input[starting_pos..starting_pos + 1] == b"e" {
            starting_pos += 1;
            break;
        }

        let (key, offset) = decode_string(&to_vec(&input[starting_pos..]));
        starting_pos += offset as usize;

        let (value, offset) = decode(&to_vec(&input[starting_pos..]));
        starting_pos += offset as usize;
        decoded.insert(key, value);
    }

    (decoded, starting_pos as u32)
}
