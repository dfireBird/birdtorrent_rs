use crate::bencoding;
use crate::bencoding::{BDict, BInt, BList, BString, BType};
use crate::torrent::Torrent;
use crate::utility;
use crate::utility::{PeerId, PORT};

use url::form_urlencoded;

use std::borrow::Cow;
use std::convert::TryInto;
use std::net::Ipv4Addr;

#[derive(Debug)]
pub struct Peer {
    ip: Ipv4Addr,
    port: u16,
}

#[derive(Debug)]
pub struct TrackerResponse {
    interval: i64,
    complete: i64,
    incomplete: i64,
    peer_list: Vec<Peer>,
}

pub fn announce(
    torrent_data: Torrent,
    info: &BDict,
    uploaded: i64,
    downloaded: i64,
    left: i64,
    event: Option<&str>,
) -> TrackerResponse {
    let mut announce_url = torrent_data.get_announce();
    let info_hash = utility::hash(info.encode());

    let query = create_tracker_query(info_hash, uploaded, downloaded, left, event);

    announce_url.push('?');
    announce_url.push_str(&query);

    let (tracker_response, _) = bencoding::decode(&get(&announce_url));
    let tracker_response = tracker_response.as_any().downcast_ref::<BDict>().unwrap();
    parse_tracker_response(tracker_response)
}

#[tokio::main]
async fn get(url: &str) -> Vec<u8> {
    let resp = reqwest::get(url).await.unwrap();
    if resp.status() != 200 {
        panic!(
            "Status Code: {}, Error: {}",
            resp.status(),
            resp.text().await.unwrap()
        );
    }

    resp.bytes().await.unwrap().to_vec()
}

fn create_tracker_query(
    info_hash: Vec<u8>,
    uploaded: i64,
    downloaded: i64,
    left: i64,
    event: Option<&str>,
) -> String {
    let mut PEER_ID = PeerId::new();
    let mut query = form_urlencoded::Serializer::new(String::new());
    query
        .append_pair("peer_id", &PEER_ID.value())
        .append_pair("port", &PORT.to_string())
        .append_pair("uploaded", &uploaded.to_string())
        .append_pair("downloaded", &downloaded.to_string())
        .append_pair("left", &left.to_string())
        .append_pair("compact", "1");

    match event {
        Some(event_id) => {
            match event_id {
                "started" | "completed" | "stopped" => query.append_pair("event", event_id),
                _ => &mut form_urlencoded::Serializer::new(String::new()),
            };
            ()
        }
        None => (),
    }

    let query = query
        .encoding_override(Some(&|input| {
            if input != "!" {
                Cow::Borrowed(input.as_bytes())
            } else {
                Cow::Owned(info_hash.clone())
            }
        }))
        .append_pair("info_hash", "!")
        .finish();
    query
}

fn parse_tracker_response(tracker_response: &BDict) -> TrackerResponse {
    match tracker_response.get::<BString>("failure reason") {
        Some(reason) => panic!(
            "Tracker response failure: {}",
            reason.into_string().unwrap()
        ),
        None => (),
    }

    let interval = tracker_response.get::<BInt>("interval").unwrap().into_int();
    let complete = tracker_response.get::<BInt>("complete").unwrap().into_int();
    let incomplete = tracker_response
        .get::<BInt>("incomplete")
        .unwrap()
        .into_int();

    let peer_list = tracker_response.get::<BString>("peers").unwrap().to_vec();
    TrackerResponse {
        interval,
        complete,
        incomplete,
        peer_list: parse_peers_string(peer_list),
    }
}

fn parse_peers_string(peers_string: Vec<u8>) -> Vec<Peer> {
    if peers_string.len() % 6 != 0 {
        panic!("Invalid Peer String");
    }

    let mut peer_list = Vec::new();

    let mut i = 0;
    while i < peers_string.len() {
        let ip: [u8; 4] = peers_string[i..i + 4].try_into().unwrap();
        let port: [u8; 2] = peers_string[i + 4..i + 6].try_into().unwrap();
        let ip = Ipv4Addr::from(ip);
        let port = u16::from_be_bytes(port);
        peer_list.push(Peer { ip, port });
        i += 6;
    }
    peer_list
}
