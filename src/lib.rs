mod bencoding;
mod message;
mod p2p;
mod torrent;
mod tracker;
mod utility;

/***

//without upload
fn start_transfer(file_name: String) {
    if !is_torrentFile() {
        eprintln!("File given is not a .torrent file");
        std::process::exit(1);
    }

    let session = Session::new();
    let peer_id = utility::PeerId::new();

    let torrent_file = std::fs::read(file_name); //Handle the possible errors

    let (torrent_file, _) = bencoding::decode(torrent_file);
    let torrent_file = torrent_file.as_any().downcast_ref::<BDict>().unwrap();

    let torrent_data = torrent::parse_torrent_data(torrent_file);

    let info_hash = generate_info_hash(torrent_file);

    let tracker_response = tracker::announce(
        torrent_data,
        info_hash,
        peer_id,
        session.get_downloaded(),
        session.get_uploaded(),
        torrent_data.get_length(),
        Some("started"),
    );

    let thread_id = Vec::new();
    for peer in tracker_response.get_peers() {
        thread_id.push(std::thread::spawn(|| {
            let connection = p_to_p::iniate_connection;
            loop {
                if is_completed() || is_paused() {
                    connection.close();
                    break;
                }
                connection.download_piece(torrent_data);
            }
        }));
    }

    for thread in thread_id {
        thread.join();
    }

    tracker::announce(torrent_data, info_hash, peer_id, session.get_downloaded(), session.get_uploaded(), torrent_data.get_length(), Some("stopped"));
}

***/
