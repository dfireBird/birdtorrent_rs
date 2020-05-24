use crate::client::{Client, MessageReturn};
use crate::message::MessageId;
use crate::torrent::Torrent;
use crate::utility;

use std::{thread, time};

const MAX_REQ_SIZE: i64 = 16384;

pub fn download_piece(client: &mut Client, index: u32, piece_length: i64) -> Vec<u8> {
    let mut no_of_iterations = piece_length / MAX_REQ_SIZE;
    if piece_length % MAX_REQ_SIZE != 0 {
        no_of_iterations += 1;
    }
    let mut piece_length = piece_length;
    let mut begin = 0i64;
    let mut piece: Vec<u8> = Vec::new();
    for _i in 0..no_of_iterations {
        if piece_length > MAX_REQ_SIZE {
            client.send_request(index, begin as u32, MAX_REQ_SIZE as u32);
            piece_length -= MAX_REQ_SIZE;
            begin += MAX_REQ_SIZE;
        } else {
            client.send_request(index, begin as u32, piece_length as u32);
            piece_length -= piece_length;
            begin += piece_length;
        }

        thread::sleep(time::Duration::from_secs(3));
        let (id, payload) = match client.receive_message() {
            (MessageId::KeepAlive, _) => client.receive_message(),
            (id, payload) => (id, payload),
        };

        if let MessageId::Piece = id {
            let mut block = if let MessageReturn::Piece((_, _, block)) = payload.unwrap() {
                block
            } else {
                panic!("Wrong Message returned");
            };

            piece.append(&mut block);
        }
    }

    piece
}

pub fn check_intergrity(torrent: &Torrent, index: u32, piece: Vec<u8>) -> bool {
    let orginal_hash = torrent.get_piece_hash(index);
    let piece_hash = utility::hash(piece);

    if piece_hash == orginal_hash {
        true
    } else {
        false
    }
}
