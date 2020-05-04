pub mod btype;

use btype::*;

use std::collections::HashMap;

pub fn decode(input: &str) -> Box<dyn BType> {
    let cur = input.as_bytes()[0];
    let mut result: Box<dyn BType>;
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
    BString::new("Hello")
}

fn decode_int(input: &str) -> BInt {
    BInt::new(23)
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
