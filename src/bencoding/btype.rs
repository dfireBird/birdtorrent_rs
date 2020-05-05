use std::collections::HashMap;
use std::fmt::Debug;

pub trait BType: Debug {
    fn encode(&self) -> String;
}

#[derive(Debug)]
pub struct BInt(i64);

impl BInt {
    pub fn new(data: i64) -> BInt {
        BInt(data)
    }
}

impl BType for BInt {
    fn encode(&self) -> String {
        format!("i{}e", self.0)
    }
}

#[derive(Debug, Eq, Hash, PartialEq)]
pub struct BString(String);

impl BString {
    pub fn new(data: &str) -> BString {
        BString(data.to_string())
    }
}

impl BType for BString {
    fn encode(&self) -> String {
        format!("{}:{}", self.0.len(), self.0)
    }
}

#[derive(Debug)]
pub struct BList(Vec<Box<dyn BType>>);

impl BList {
    pub fn new(data: Vec<Box<dyn BType>>) -> BList {
        BList(data)
    }

    pub fn push(&mut self, data: Box<dyn BType>) {
        self.0.push(data);
    }
}

impl BType for BList {
    fn encode(&self) -> String {
        let mut encoded = String::from("l");
        for contents in &self.0 {
            encoded.push_str(&(contents.encode()));
        }
        encoded.push('e');
        encoded
    }
}

#[derive(Debug)]
pub struct BDict(HashMap<BString, Box<dyn BType>>);

impl BDict {
    pub fn new(data: HashMap<BString, Box<dyn BType>>) -> BDict {
        BDict(data)
    }

    pub fn insert(&mut self, key: BString, value: Box<dyn BType>) {
        self.0.insert(key, value);
    }
}

impl BType for BDict {
    fn encode(&self) -> String {
        let mut encoded = String::from("d");
        for (key, val) in &self.0 {
            encoded.push_str(&key.encode());
            encoded.push_str(&val.encode());
        }
        encoded.push('e');
        encoded
    }
}
