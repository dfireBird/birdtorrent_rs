use super::{Peer, TrackerResponse};
use crate::utility::{PeerId, PORT};

use rand::Rng;
use url::Url;

use std::convert::TryInto;
use std::net::ToSocketAddrs;
use std::net::{Ipv4Addr, UdpSocket};

pub fn announce(
    announce_url: Url,
    info_hash: &Vec<u8>,
    peer_id: &mut PeerId,
    uploaded: i64,
    downloaded: i64,
    left: i64,
    event: Option<&str>,
) -> TrackerResponse {
    let mut socket = UdpSocket::bind("0.0.0.0:0").unwrap();

    let ip = (
        announce_url.host_str().unwrap(),
        announce_url.port().unwrap(),
    );
    socket.connect(ip).unwrap();

    let transaction_id = rand::thread_rng().gen();
    socket.send(&build_connect_req(transaction_id)).unwrap();

    let mut resp_buffer = [0u8; 65535];
    let length = socket.recv(&mut resp_buffer).unwrap();

    let connection_id = parse_connect_resp(&resp_buffer, length, transaction_id);

    let transaction_id = rand::thread_rng().gen();
    socket
        .send(&build_announce_req(
            connection_id,
            transaction_id,
            info_hash,
            peer_id,
            uploaded,
            downloaded,
            left,
            event,
        ))
        .unwrap();

    let length = socket.recv(&mut resp_buffer).unwrap();
    parse_announce_resp(&resp_buffer, length, transaction_id)
}

fn build_announce_req(
    connection_id: u64,
    transaction_id: u32,
    info_hash: &Vec<u8>,
    peer_id: &mut PeerId,
    uploaded: i64,
    downloaded: i64,
    left: i64,
    event: Option<&str>,
) -> Vec<u8> {
    let event = match event {
        Some("completed") => 1u32,
        Some("started") => 2,
        Some("stopped") => 3,
        _ => 0,
    };
    let mut buffer = vec![];

    buffer.append(&mut connection_id.to_be_bytes().to_vec());
    buffer.append(&mut 1u32.to_be_bytes().to_vec()); //action
    buffer.append(&mut transaction_id.to_be_bytes().to_vec()); //transaction id
    buffer.append(&mut info_hash.to_vec());
    buffer.append(&mut peer_id.value().as_bytes().to_vec());
    buffer.append(&mut downloaded.to_be_bytes().to_vec());
    buffer.append(&mut left.to_be_bytes().to_vec());
    buffer.append(&mut uploaded.to_be_bytes().to_vec());
    buffer.append(&mut event.to_be_bytes().to_vec());
    buffer.append(&mut 0u32.to_be_bytes().to_vec()); //Ip Address
    buffer.append(&mut rand::thread_rng().gen::<u32>().to_be_bytes().to_vec()); //key
    buffer.append(&mut (-1i32).to_be_bytes().to_vec());
    buffer.append(&mut (PORT as u16).to_be_bytes().to_vec());

    buffer
}

fn parse_announce_resp(buffer: &[u8], length: usize, transaction_id: u32) -> TrackerResponse {
    if length < 20 {
        panic!("Length of bytes recieved is less than 20 bytes (must be aleast 20 bytes)");
    }
    let action = u32::from_be_bytes(buffer[0..4].try_into().unwrap());
    if action != 1 {
        panic!("Wrong action value recieved");
    }
    let r_transaction_id = u32::from_be_bytes(buffer[4..8].try_into().unwrap());
    if r_transaction_id != transaction_id {
        panic!("Wrong transaction id recieved");
    }

    let interval = u32::from_be_bytes(buffer[8..12].try_into().unwrap());
    let incomplete = u32::from_be_bytes(buffer[12..16].try_into().unwrap());
    let complete = u32::from_be_bytes(buffer[16..20].try_into().unwrap());

    let length = length - 20;
    let buffer = &buffer[20..];

    let mut i = 0;
    let mut peer_list = Vec::new();

    while i < length {
        let ip: [u8; 4] = buffer[i..i + 4].try_into().unwrap();
        let ip = Ipv4Addr::from(ip);
        let port = u16::from_be_bytes(buffer[i + 4..i + 6].try_into().unwrap());
        peer_list.push(Peer { ip, port });
        i += 6;
    }

    TrackerResponse {
        interval,
        complete,
        incomplete,
        peer_list,
    }
}

fn build_connect_req(transaction_id: u32) -> Vec<u8> {
    let mut buffer = vec![];
    buffer.append(&mut 0x41727101980u64.to_be_bytes().to_vec());
    buffer.append(&mut 0u32.to_be_bytes().to_vec()); //action
    buffer.append(&mut transaction_id.to_be_bytes().to_vec()); //transaction id
    buffer
}

fn parse_connect_resp(buffer: &[u8], length: usize, transaction_id: u32) -> u64 {
    if length < 16 {
        panic!("Length of bytes recieved is less than 16 bytes (must be atleast 16 bytes)")
    }
    let action = u32::from_be_bytes(buffer[0..4].try_into().unwrap());
    if action != 0 {
        panic!("Wrong action value recieved");
    }
    let r_transaction_id = u32::from_be_bytes(buffer[4..8].try_into().unwrap());
    if r_transaction_id != transaction_id {
        panic!("Wrong transaction id recieved");
    }
    let connection_id = u64::from_be_bytes(buffer[8..16].try_into().unwrap());
    connection_id
}
