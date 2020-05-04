pub mod btype;

use btype::{BDict, BInt, BList, BString, BType};

use std::collections::HashMap;

pub fn decode(input: &str) -> Box<dyn BType> {
    let cur = input.as_bytes()[0];
    let result: Box<dyn BType>;
    if cur.is_ascii_digit() {
        result = Box::new(decode_string(input));
    } else {
        match cur {
            b'd' => {
                result = Box::new(decode_dict(input));
            }
            b'l' => {
                result = Box::new(decode_list(input));
            }
            b'i' => {
                result = Box::new(decode_int(input));
            }
            _ => panic!("Not a valid type"),
        }
    }
    result
}

fn decode_string(input: &str) -> BString {
    let mut delimiter_pos = input.find(':').unwrap();

    let length: usize = input[delimiter_pos - 1..delimiter_pos].parse().unwrap();
    delimiter_pos += 1;
    let result = input.get(delimiter_pos..delimiter_pos + length).unwrap();
    BString::new(result)
}

fn decode_int(input: &str) -> BInt {
    let starting_pos = input.find('i').unwrap() + 1;
    let delimiter_pos = input.find('e').unwrap();

    let result = input[starting_pos..delimiter_pos].parse().unwrap();
    BInt::new(result)
}

fn decode_list(input: &str) -> BList {
    BList::new(vec![
        Box::new(BInt::new(23)),
        Box::new(BString::new("Hello")),
    ])
}

fn decode_dict(input: &str) -> BDict {
    BDict::new(HashMap::new())
}
