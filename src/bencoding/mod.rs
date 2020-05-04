pub mod btype;

use btype::{BDict, BInt, BList, BString, BType};

use std::collections::HashMap;

pub fn decode(input: &str) -> (Box<dyn BType>, u32) {
    let cur = input.as_bytes()[0];
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

fn decode_string(input: &str) -> (BString, u32) {
    let mut delimiter_pos = input.find(':').unwrap();

    let length: usize = input[delimiter_pos - 1..delimiter_pos].parse().unwrap();
    delimiter_pos += 1;
    let result = input.get(delimiter_pos..delimiter_pos + length).unwrap();
    (BString::new(result), (delimiter_pos + length) as u32)
}

fn decode_int(input: &str) -> (BInt, u32) {
    let starting_pos = input.find('i').unwrap() + 1;
    let delimiter_pos = input.find('e').unwrap();

    let result = input[starting_pos..delimiter_pos].parse().unwrap();
    (BInt::new(result), (delimiter_pos + 1) as u32)
}

fn decode_list(input: &str) -> (BList, u32) {
    let mut starting_pos = input.find('l').unwrap() + 1;
    let last_char_pos = input.len();
    let mut decoded: BList = BList::new(Vec::new());

    while starting_pos < last_char_pos {
        if &input[starting_pos..starting_pos + 1] == "e" {
            starting_pos += 1;
            continue;
        }

        let (result, offset) = decode(&input[starting_pos..]);
        decoded.push(result);
        starting_pos += offset as usize;
    }

    (decoded, (last_char_pos) as u32)
}

fn decode_dict(input: &str) -> (BDict, u32) {
    (BDict::new(HashMap::new()), 32)
}
