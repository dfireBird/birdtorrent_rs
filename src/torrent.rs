use crate::bencoding::btype::{BDict, BInt, BList, BString};

#[derive(Debug)]
struct SingleFileMetaInfo {
    info: SingleFileInfo,
    announce: String,
}

#[derive(Debug)]
struct SingleFileInfo {
    name: String,
    length: i64,
    piece_length: i64,
    pieces: Vec<[u8; 20]>,
}

#[derive(Debug)]
struct MultiFileMetaInfo {
    info: MultiFileInfo,
    announce: String,
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

enum Torrent {
    SingleFileTorrent(SingleFileMetaInfo),
    MultiFileTorrent(MultiFileMetaInfo),
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
    let pieces = Vec::new();

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
