mod http;
mod udp;

use crate::utility::PeerId;

use url::Url;

use std::net::{IpAddr, Ipv4Addr};

#[derive(Clone, Debug)]
pub struct Peer {
    ip: Ipv4Addr,
    port: u16,
}

impl Peer {
    pub fn get_ip(&self) -> IpAddr {
        IpAddr::V4(self.ip)
    }

    pub fn get_port(&self) -> u16 {
        self.port
    }
}

#[derive(Debug)]
pub struct TrackerResponse {
    interval: i64,
    complete: i64,
    incomplete: i64,
    peer_list: Vec<Peer>,
}

impl TrackerResponse {
    pub fn get_peer_list(&self) -> Vec<Peer> {
        self.peer_list.to_vec()
    }
}

pub fn announce(
    announce_url: String,
    info_hash: &Vec<u8>,
    peer_id: &mut PeerId,
    uploaded: u32,
    downloaded: u32,
    left: u32,
    event: Option<&str>,
) -> TrackerResponse {
    let announce_url = Url::parse(&announce_url).unwrap();

    match announce_url.scheme() {
        "http" | "https" => http::announce(
            announce_url.as_str(),
            info_hash,
            peer_id,
            uploaded,
            downloaded,
            left,
            event,
        ),
        "udp" => udp::announce(
            announce_url,
            info_hash,
            peer_id,
            uploaded,
            downloaded,
            left,
            event,
        ),
        _ => panic!("Invalid announce url"),
    }
}
