use std::any::Any;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::str;

use crate::utility::to_vec;

pub trait BType: Debug {
    fn encode(&self) -> Vec<u8>;
    fn as_any(&self) -> &dyn Any;
}

#[derive(Debug)]
pub struct BInt(i64);

impl BInt {
    pub fn new(data: i64) -> BInt {
        BInt(data)
    }

    pub fn into_int(&self) -> i64 {
        self.0
    }
}

impl BType for BInt {
    fn encode(&self) -> Vec<u8> {
        format!("i{}e", self.0).as_bytes().to_vec()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct BString(Vec<u8>);

impl BString {
    pub fn new(data: &Vec<u8>) -> BString {
        BString(data.clone())
    }

    pub fn into_string(&self) -> Option<String> {
        match str::from_utf8(&self.0) {
            Ok(value) => Some(String::from(value)),
            Err(_) => None,
        }
    }
}

impl BType for BString {
    fn encode(&self) -> Vec<u8> {
        let mut encoded = format!("{}:", self.0.len()).as_bytes().to_vec();
        encoded.append(&mut self.0.clone());
        encoded
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl std::ops::Deref for BString {
    type Target = Vec<u8>;
    fn deref(&self) -> &Vec<u8> {
        &self.0
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

    pub fn get(&self) -> &Vec<Box<dyn BType>> {
        &self.0
    }
}

impl BType for BList {
    fn encode(&self) -> Vec<u8> {
        let mut encoded = String::from("l").as_bytes().to_vec();
        for contents in &self.0 {
            encoded.append(&mut contents.encode());
        }
        encoded.push(b'e');
        encoded
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug)]
pub struct BDict(BTreeMap<BString, Box<dyn BType>>);

impl BDict {
    pub fn new(data: BTreeMap<BString, Box<dyn BType>>) -> BDict {
        BDict(data)
    }

    pub fn insert(&mut self, key: BString, value: Box<dyn BType>) {
        self.0.insert(key, value);
    }

    pub fn get<T: 'static + BType>(&self, key: &str) -> Option<&T> {
        match self.0.get(&BString::new(&to_vec(key.as_bytes()))) {
            Some(value) => Some(value.as_any().downcast_ref::<T>().unwrap()),
            None => None,
        }
    }
}

impl BType for BDict {
    fn encode(&self) -> Vec<u8> {
        let mut encoded = String::from("d").as_bytes().to_vec();
        for (key, val) in &self.0 {
            encoded.append(&mut key.encode());
            encoded.append(&mut val.encode());
        }
        encoded.push(b'e');
        encoded
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
