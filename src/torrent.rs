use crate::bencoding::{BDict, BInt, BList, BString};

use std::convert::TryInto;

#[derive(Debug)]
pub struct SingleFileMetaInfo {
    info: SingleFileInfo,
    announce: String,
    pieces: Vec<u8>,
}

#[derive(Debug)]
struct SingleFileInfo {
    name: String,
    length: i64,
    piece_length: i64,
    pieces: Vec<[u8; 20]>,
}

#[derive(Debug)]
pub struct MultiFileMetaInfo {
    info: MultiFileInfo,
    announce: String,
    pieces: Vec<u8>,
}

#[derive(Debug)]
struct MultiFileInfo {
    name: String,
    files: Vec<File>,
    piece_length: i64,
    pieces: Vec<[u8; 20]>,
}

#[derive(Debug)]
struct File {
    length: i64,
    path: Vec<String>,
}

#[derive(Debug)]
pub enum Torrent {
    SingleFileTorrent(SingleFileMetaInfo),
    MultiFileTorrent(MultiFileMetaInfo),
}

impl Torrent {
    pub fn get_announce(&self) -> String {
        let announce = match self {
            Torrent::SingleFileTorrent(meta_data) => &meta_data.announce,
            Torrent::MultiFileTorrent(meta_data) => &meta_data.announce,
        };

        String::from(announce)
    }

    pub fn get_length(&self) -> i64 {
        match self {
            Torrent::SingleFileTorrent(meta_data) => meta_data.info.length,
            Torrent::MultiFileTorrent(meta_data) => {
                let mut length = 0i64;
                for file in &meta_data.info.files {
                    length += file.length;
                }
                length
            }
        }
    }

    pub fn set_piece(&mut self, index: u32) {
        match self {
            Torrent::MultiFileTorrent(meta_data) => meta_data.pieces[index as usize] = 1,

            Torrent::SingleFileTorrent(meta_data) => meta_data.pieces[index as usize] = 1,
        }
    }
}

pub fn parse_torrent_data(torrent_meta_data: &BDict) -> Torrent {
    let announce = torrent_meta_data
        .get::<BString>("announce")
        .unwrap()
        .into_string()
        .unwrap();
    let info = torrent_meta_data.get::<BDict>("info").unwrap();

    let name = info.get::<BString>("name").unwrap().into_string().unwrap();
    let piece_length = info.get::<BInt>("piece length").unwrap().into_int();
    let pieces = make_pieces(&info.get::<BString>("pieces").unwrap().to_vec());

    let torrent: Torrent;
    match info.get::<BList>("files") {
        Some(file_list) => {
            let file_list = file_list.get();
            let mut files: Vec<File> = Vec::new();
            for file in file_list {
                let file = file.as_any().downcast_ref::<BDict>().unwrap();
                let length = file.get::<BInt>("length").unwrap().into_int();
                let path_list = file.get::<BList>("path").unwrap().get().clone();
                let mut path: Vec<String> = Vec::new();
                for paths in path_list {
                    let paths = paths
                        .as_any()
                        .downcast_ref::<BString>()
                        .unwrap()
                        .into_string()
                        .unwrap();
                    path.push(paths);
                }

                files.push(File { path, length });
            }

            torrent = Torrent::MultiFileTorrent(MultiFileMetaInfo {
                announce,
                pieces: vec![0; piece_length as usize],
                info: MultiFileInfo {
                    name,
                    files,
                    piece_length,
                    pieces,
                },
            });
        }

        None => {
            let length = info.get::<BInt>("length").unwrap().into_int();

            torrent = Torrent::SingleFileTorrent(SingleFileMetaInfo {
                announce,
                pieces: vec![0; piece_length as usize],
                info: SingleFileInfo {
                    name,
                    length,
                    piece_length,
                    pieces,
                },
            });
        }
    };

    torrent
}

fn make_pieces(pieces: &Vec<u8>) -> Vec<[u8; 20]> {
    let mut pieces_array = Vec::new();

    let mut i = 0;
    while i < pieces.len() {
        let single_piece: [u8; 20] = pieces[i..i + 20].try_into().unwrap();
        pieces_array.push(single_piece);
        i += 20
    }

    pieces_array
}
