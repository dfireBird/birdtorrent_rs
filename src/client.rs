use crate::message;
use crate::message::MessageId;
use crate::tracker::Peer;
use crate::utility::encode_hex;

use std::io::prelude::*;
use std::net::{SocketAddr, TcpStream};

#[allow(non_upper_case_globals)]
const pstr: &str = "BitTorrent protocol";

pub struct Client {
    info_hash: Vec<u8>,
    peer_id: String,
    peer: Peer,
    choked: bool,
    connection: TcpStream,
}

impl Client {
    pub fn new(info_hash: &Vec<u8>, peer_id: &str, peer: Peer) -> Option<Client> {
        let socket = SocketAddr::new(peer.get_ip(), peer.get_port());
        let mut connection = TcpStream::connect(socket).unwrap();
        let mut client = Client {
            info_hash: info_hash.to_vec(),
            peer_id: peer_id.to_string(),
            choked: true,
            peer,
            connection,
        };

        match client.handshake() {
            Ok(_) => Some(client),
            Err(error) => {
                eprintln!("{}", error);
                None
            }
        }
    }

    pub fn send_interested(&mut self) {
        self.connection
            .write(&message::serialize_message(MessageId::Interested, None))
            .unwrap();
    }

    pub fn send_request(&mut self, index: u32, begin: u32, length: u32) {
        let mut payload = Vec::new();
        payload.append(&mut index.to_be_bytes().to_vec());
        payload.append(&mut begin.to_be_bytes().to_vec());
        payload.append(&mut length.to_be_bytes().to_vec());

        self.connection
            .write(&message::serialize_message(
                MessageId::Request,
                Some(payload),
            ))
            .unwrap();
    }

    pub fn send_have(&mut self, index: u32) {
        self.connection
            .write(&message::serialize_message(
                MessageId::Have,
                Some(index.to_be_bytes().to_vec()),
            ))
            .unwrap();
    }

    fn handshake(&mut self) -> Result<(), String> {
        let message = handshake_serialize(self);
        &mut self.connection.write(&message).unwrap();

        let mut response = [0; 49 + pstr.len()];
        &mut self.connection.read(&mut response).unwrap();

        let recieved_pstr_len = response[0];
        let offset = (recieved_pstr_len + 9) as usize;
        let recieved_info_hash = response[offset..offset + 20].to_vec();

        if self.info_hash != recieved_info_hash {
            return Err(format!(
                "Recieved info hash({}) does not match the client info hash({})",
                encode_hex(&self.info_hash),
                encode_hex(&recieved_info_hash)
            ));
        }

        fn handshake_serialize(client: &Client) -> Vec<u8> {
            let mut message: Vec<u8> = Vec::new();

            message.push(pstr.len() as u8);
            message.append(&mut pstr.as_bytes().to_vec());
            message.append(&mut vec![0; 8]); //reserved
            message.append(&mut client.info_hash.to_vec());
            message.append(&mut client.peer_id.as_bytes().to_vec());
            message
        }

        Ok(())
    }
}
