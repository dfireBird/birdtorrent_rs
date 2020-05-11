use sha1::{Digest, Sha1};
use std::time::{SystemTime, UNIX_EPOCH};

pub const PORT: i32 = 6882;

pub fn hash(input: Vec<u8>) -> Vec<u8> {
    let mut hasher = Sha1::new();
    hasher.input(input);
    let result = hasher.result();
    to_vec(&result[..])
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
