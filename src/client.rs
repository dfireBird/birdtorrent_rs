use crate::message;
use crate::message::MessageId;
use crate::tracker::Peer;
use crate::utility::encode_hex;

use std::io::prelude::*;
use std::net::{SocketAddr, TcpStream};

#[allow(non_upper_case_globals)]
const pstr: &str = "BitTorrent protocol";

pub enum MessageReturn {
    Have(u32),
    Piece((u32, u32, Vec<u8>)),
}

#[derive(Debug)]
pub struct Client {
    info_hash: Vec<u8>,
    peer_id: String,
    peer: Peer,
    choked: bool,
    connection: TcpStream,
    bitfield: Option<Vec<u8>>,
}

impl Client {
    pub fn new(info_hash: &Vec<u8>, peer_id: &str, peer: Peer) -> Option<Client> {
        let socket = SocketAddr::new(peer.get_ip(), peer.get_port());
        let mut connection =
            match TcpStream::connect_timeout(&socket, std::time::Duration::new(10, 0)) {
                Ok(connection) => connection,
                Err(_) => return None,
            };
        let mut client = Client {
            info_hash: info_hash.to_vec(),
            peer_id: peer_id.to_string(),
            choked: true,
            peer,
            connection,
            bitfield: None,
        };

        match client.handshake() {
            Ok(_) => {
                client.bitfield = client.receive_bitfield();
                Some(client)
            }
            Err(error) => {
                eprintln!("Error: {}", error);
                None
            }
        }
    }

    pub fn is_choked(&mut self) -> bool {
        self.choked
    }

    pub fn has_piece(&mut self, index: u32) -> bool {
        match &self.bitfield {
            Some(bitfield) if bitfield[index as usize] == 1 => true,
            _ => false,
        }
    }

    pub fn receive_message(&mut self) -> (MessageId, Option<MessageReturn>) {
        let mut length = [0u8; 4];
        self.connection.read(&mut length).unwrap();
        let length = u32::from_be_bytes(length);
        if length == 0 {
            return (MessageId::KeepAlive, None);
        }
        let mut message_id = [0u8; 1];
        self.connection.read(&mut message_id).unwrap();
        let message_id = MessageId::new(message_id[0] as u32);
        match message_id {
            MessageId::UnChoke => {
                self.set_choked(false);
                (MessageId::UnChoke, None)
            }
            MessageId::Choke => {
                self.set_choked(true);
                (MessageId::Choke, None)
            }
            MessageId::Have => (
                MessageId::Have,
                Some(MessageReturn::Have(self.receive_have())),
            ),
            MessageId::Piece => (
                MessageId::Piece,
                Some(MessageReturn::Piece(self.receive_piece(length))),
            ),
            _ => (MessageId::Invalid, None),
        }
    }

    pub fn send_have(&mut self, index: u32) {
        self.connection
            .write(&message::serialize_message(
                MessageId::Have,
                Some(index.to_be_bytes().to_vec()),
            ))
            .unwrap();
    }

    pub fn send_interested(&mut self) {
        self.connection
            .write(&message::serialize_message(MessageId::Interested, None))
            .unwrap();
    }

    pub fn send_request(&mut self, index: u32, begin: u32, length: u32) -> usize {
        if self.choked {
            return 0;
        }
        let mut payload = Vec::new();
        payload.append(&mut index.to_be_bytes().to_vec());
        payload.append(&mut begin.to_be_bytes().to_vec());
        payload.append(&mut length.to_be_bytes().to_vec());

        self.connection
            .write(&message::serialize_message(
                MessageId::Request,
                Some(payload),
            ))
            .unwrap()
    }

    fn handshake(&mut self) -> Result<(), String> {
        let message = handshake_serialize(self);
        &mut self.connection.write(&message).unwrap();

        let mut response = [0; 49 + pstr.len()];
        &mut self.connection.read(&mut response).unwrap();

        let received_pstr_len = response[0];
        let offset = (received_pstr_len + 9) as usize;
        let received_info_hash = response[offset..offset + 20].to_vec();

        if self.info_hash != received_info_hash {
            return Err(format!(
                "Received info hash({}) does not match the client info hash({})",
                encode_hex(&self.info_hash),
                encode_hex(&received_info_hash)
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

    fn receive_bitfield(&mut self) -> Option<Vec<u8>> {
        let mut length = [0; 4];
        self.connection.read(&mut length).unwrap();
        let length = u32::from_be_bytes(length);
        if length == 0 {
            return None;
        }

        let mut message_id = [0; 1];
        self.connection.read(&mut message_id).unwrap();
        let message_id = message_id[0];
        if message_id != 5 {
            return None;
        }

        let mut bitfield = Vec::new();
        let mut bitfield_buff = vec![0; (length - 1) as usize];
        self.connection.read(&mut bitfield_buff).unwrap();

        for mut byte in bitfield_buff {
            let mut bin = vec![0; 8];
            let mut index = 8i8 - 1;

            while index >= 0 {
                bin[index as usize] = byte & 1;
                index -= 1;
                byte >>= 1;
            }

            bitfield.append(&mut bin);
        }

        Some(bitfield)
    }

    fn receive_have(&mut self) -> u32 {
        let mut index = [0; 4];
        self.connection.read(&mut index).unwrap();
        let index = u32::from_be_bytes(index);
        index
    }

    fn receive_piece(&mut self, length: u32) -> (u32, u32, Vec<u8>) {
        let mut buff = [0; 4];
        self.connection.read(&mut buff).unwrap();
        let index = u32::from_be_bytes(buff);
        self.connection.read(&mut buff).unwrap();
        let begin = u32::from_be_bytes(buff);
        let mut block = vec![0u8; (length - 9) as usize];
        self.connection.read(&mut block).unwrap();
        (index, begin, block)
    }

    fn set_choked(&mut self, state: bool) {
        self.choked = state;
    }
}
