use crate::torrent::{File, Torrent};

use std::fs::{self, OpenOptions};
use std::io::prelude::*;
use std::io::{self, SeekFrom};

pub fn write_piece(piece: Vec<u8>, index: u32, meta_data: &Torrent) {
    match &meta_data {
        Torrent::SingleFileTorrent(single_data) => write_piece_single_file(
            piece,
            index,
            meta_data.get_piece_length(),
            single_data.get_name(),
            meta_data.get_length(),
        ),
        Torrent::MultiFileTorrent(multi_data) => write_piece_multi_file(
            piece,
            index,
            meta_data.get_piece_length(),
            multi_data.get_name(),
            multi_data.get_files(index),
        ),
    }
}

fn write_piece_single_file(piece: Vec<u8>, index: u32, piece_length: i64, name: &str, length: i64) {
    let mut file = match OpenOptions::new().write(true).open(name) {
        Ok(file) => file,
        Err(err) => {
            if let io::ErrorKind::NotFound = err.kind() {
                fs::write(name, vec![0u8; length as usize]).unwrap();
            }
            OpenOptions::new().write(true).open(name).unwrap()
        }
    };
    file.seek(SeekFrom::Start((index as i64 * piece_length) as u64))
        .unwrap();
    file.write_all(&piece).unwrap();
}

fn write_piece_multi_file(
    piece: Vec<u8>,
    index: u32,
    piece_length: i64,
    name: &str,
    files: Vec<File>,
) {
    let mut piece = piece.clone();
    for file_data in files {
        let start = (index - file_data.get_start_index()) as i64 * piece_length;
        let length = file_data.get_length() - start;

        let path = file_data.get_path();
        fs::create_dir_all(format!("{}/{}", name, path[..path.len() - 1].join("/"))).unwrap();
        let filename = &path[path.len() - 1];

        let mut file = match OpenOptions::new().write(true).open(filename) {
            Ok(file) => file,
            Err(err) => {
                if let io::ErrorKind::NotFound = err.kind() {
                    fs::write(filename, vec![0u8; file_data.get_length() as usize]).unwrap();
                }
                OpenOptions::new().write(true).open(filename).unwrap()
            }
        };

        if length >= piece_length {
            file.seek(SeekFrom::Start(start as u64)).unwrap();
            file.write_all(&piece).unwrap();
        } else {
            let length = length as usize;
            file.seek(SeekFrom::Start(start as u64)).unwrap();
            let to_be_write: Vec<u8> = piece.drain(..length as usize).collect();
            file.write_all(&to_be_write);
        }
    }
}
