use crate::bencoding::{BDict, BInt, BList, BString};

use rand::Rng;
use std::convert::TryInto;

#[derive(Clone, Debug)]
pub struct SingleFileMetaInfo {
    info: SingleFileInfo,
    announce: String,
    pieces: Vec<u8>,
}

#[derive(Clone, Debug)]
struct SingleFileInfo {
    name: String,
    length: i64,
    piece_length: i64,
    pieces: Vec<[u8; 20]>,
}

impl SingleFileMetaInfo {
    pub fn get_name(&self) -> &str {
        &self.info.name
    }
}

#[derive(Clone, Debug)]
pub struct MultiFileMetaInfo {
    info: MultiFileInfo,
    announce: String,
    pieces: Vec<u8>,
}

impl MultiFileMetaInfo {
    pub fn get_files(&self, piece_index: u32) -> Vec<File> {
        let mut files = Vec::new();
        for file in self.info.files.to_vec() {
            if file.piece_ext.0 <= piece_index && file.piece_ext.1 >= piece_index {
                files.push(file)
            }
        }
        files
    }

    pub fn get_name(&self) -> &str {
        &self.info.name
    }
}

#[derive(Clone, Debug)]
struct MultiFileInfo {
    name: String,
    files: Vec<File>,
    piece_length: i64,
    pieces: Vec<[u8; 20]>,
}

#[derive(Clone, Debug)]
pub struct File {
    length: i64,
    path: Vec<String>,
    piece_ext: (u32, u32),
}

impl File {
    pub fn get_length(&self) -> i64 {
        self.length
    }

    pub fn get_path(&self) -> Vec<String> {
        self.path.to_vec()
    }

    pub fn get_start_index(&self) -> u32 {
        self.piece_ext.0
    }

    pub fn get_end_index(&self) -> u32 {
        self.piece_ext.1
    }
}

#[derive(Clone, Debug)]
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

    pub fn get_piece_hash(&self, index: u32) -> &[u8; 20] {
        match self {
            Torrent::MultiFileTorrent(meta_data) => &meta_data.info.pieces[index as usize],
            Torrent::SingleFileTorrent(meta_data) => &meta_data.info.pieces[index as usize],
        }
    }

    pub fn get_piece_length(&self) -> i64 {
        match self {
            Torrent::MultiFileTorrent(meta_data) => meta_data.info.piece_length,
            Torrent::SingleFileTorrent(meta_data) => meta_data.info.piece_length,
        }
    }

    pub fn set_piece(&mut self, index: u32) {
        match self {
            Torrent::MultiFileTorrent(meta_data) => meta_data.pieces[index as usize] = 1,

            Torrent::SingleFileTorrent(meta_data) => meta_data.pieces[index as usize] = 1,
        }
    }

    pub fn get_piece(&self, index: u32) -> u8 {
        match self {
            Torrent::MultiFileTorrent(meta_data) => meta_data.pieces[index as usize],
            Torrent::SingleFileTorrent(meta_data) => meta_data.pieces[index as usize],
        }
    }

    pub fn generate_piece_index(&self) -> u32 {
        let total_length = match self {
            Torrent::MultiFileTorrent(meta_data) => meta_data.pieces.len(),
            Torrent::SingleFileTorrent(meta_data) => meta_data.pieces.len(),
        };
        let mut piece_index = rand::thread_rng().gen_range(0, total_length as u32);
        if self.get_piece(piece_index) == 1 {
            piece_index = self.generate_piece_index();
        }

        piece_index
    }

    pub fn is_completed(&self) -> bool {
        let pieces = match self {
            Torrent::MultiFileTorrent(meta_data) => &meta_data.pieces,
            Torrent::SingleFileTorrent(meta_data) => &meta_data.pieces,
        };

        let mut downloaded = 0;
        for piece in pieces.to_vec() {
            if piece == 1 {
                downloaded += 1;
            }
        }

        downloaded == pieces.len()
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
            let mut piece_start = 0_f32;
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
                let piece_end = length as f32 / piece_length as f32;

                files.push(File {
                    path,
                    length,
                    piece_ext: (piece_start as u32, (piece_end + piece_start) as u32),
                });

                piece_start += piece_end;
            }

            torrent = Torrent::MultiFileTorrent(MultiFileMetaInfo {
                announce,
                pieces: vec![0; pieces.len()],
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
                pieces: vec![0; pieces.len()],
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
