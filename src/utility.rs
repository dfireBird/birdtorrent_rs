use crate::bencoding::{BDict, BType};

use sha1::{Digest, Sha1};

use std::fmt::Write;
use std::time::{SystemTime, UNIX_EPOCH};

pub const PORT: i32 = 6882;

pub fn generate_info_hash(torrent: &BDict) -> Vec<u8> {
    let info = torrent.get::<BDict>("info").unwrap();
    hash(info.encode())
}

pub fn hash(input: Vec<u8>) -> Vec<u8> {
    let mut hasher = Sha1::new();
    hasher.update(input);
    let result = hasher.finalize();
    to_vec(&result[..])
}

pub fn encode_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        write!(&mut s, "{:02x}", b);
    }
    s
}

pub fn to_vec<T: Clone>(data: &[T]) -> Vec<T> {
    data.iter().cloned().collect()
}

pub struct PeerId(Option<String>);

impl PeerId {
    pub fn new() -> PeerId {
        PeerId(None)
    }
    pub fn value(&mut self) -> String {
        match &self.0 {
            Some(v) => v.to_string(),
            None => {
                let peer_id = generate_peer_id();
                self.0 = Some(peer_id.to_string());
                peer_id
            }
        }
    }
}

fn generate_peer_id() -> String {
    format!(
        "-tr0100-1{}8",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string()
    )
}
