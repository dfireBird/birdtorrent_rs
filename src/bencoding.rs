use std::collections::HashMap;

pub enum BType {
    BInt(i32),
    BString(String),
    BList(Vec<BType>),
    BDict(HashMap<BType, BType>),
}

pub fn decode(input: &String) -> BType {
    let cur = input.as_bytes()[0];
    let mut result: BType;
    if cur.is_ascii_digit() {
        result = decode_string(input);
    } else {
        match cur {
            b'd' => {
                result = decode_dict(input);
            }
            b'l' => {
                result = decode_list(input);
            }
            b'i' => {
                result = decode_int(input);
            }
            _ => result = BType::BString(String::from("Error")),
        }
    }
    result
}

fn decode_string(input: &String) -> BType {
    let mut delimiter_pos = input.find(':').unwrap();

    let length: usize = input[delimiter_pos - 1..delimiter_pos].parse().unwrap();

    delimiter_pos += 1;
    let result = input.get(delimiter_pos..delimiter_pos + length).unwrap();

    BType::BString(result.to_string())
}

fn decode_int(input: &String) -> BType {
    BType::BInt(1)
}

fn decode_list(input: &String) -> BType {
    BType::BInt(1)
}

fn decode_dict(input: &String) -> BType {
    BType::BInt(1)
}

#[cfg(test)]
pub mod bttests {
    use super::*;
    #[test]
    fn b_string() {
        let expected_result = String::from("Hello");
        let input = String::from("5:Hello");

        let result = match decode(&input) {
            BType::BString(result) => result,
            _ => String::from("Hmm"),
        };

        assert_eq!(result, expected_result);
    }
}
