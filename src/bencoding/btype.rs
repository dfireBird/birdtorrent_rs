use std::collections::HashMap;

pub trait BType {
    fn value(&self) -> String;
}

pub struct BInt(i32);

impl BInt {
    pub fn new(data: i32) -> BInt {
        BInt(data)
    }
}

impl BType for BInt {
    fn value(&self) -> String {
        format!("i{}e", self.0)
    }
}

pub struct BString(String);

impl BString {
    pub fn new(data: &str) -> BString {
        BString(data.to_string())
    }
}

impl BType for BString {
    fn value(&self) -> String {
        format!("{}:{}", self.0.len(), self.0)
    }
}

pub struct BList(Vec<Box<dyn BType>>);

impl BList {
    pub fn new(data: Vec<Box<dyn BType>>) -> BList {
        BList(data)
    }
}

impl BType for BList {
    fn value(&self) -> String {
        let mut value = String::from("l");
        for contents in &self.0 {
            value.push_str(&(contents.value()));
        }
        value.push('e');
        value
    }
}

pub struct BDict(HashMap<BString, Box<dyn BType>>);

impl BDict {
    pub fn new(data: HashMap<BString, Box<dyn BType>>) -> BDict {
        BDict(data)
    }
}

impl BType for BDict {
    fn value(&self) -> String {
        let mut value = String::from("d");
        for (key, val) in &self.0 {
            value.push_str(&key.value());
            value.push_str(&val.value());
        }
        value.push('e');
        value
    }
}
