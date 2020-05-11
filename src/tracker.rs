use crate::bencoding;
use crate::bencoding::{BDict, BInt, BList, BString, BType};
use crate::torrent::Torrent;
use crate::utility;
use crate::utility::{PeerId, PORT};

use url::form_urlencoded;

use std::borrow::Cow;

#[derive(Debug)]
pub struct Peers {
    peer_id: Vec<u8>,
    ip: String,
    port: i64,
}

#[derive(Debug)]
pub enum PeerModel {
    CompactPeer(Vec<u8>),
    VerbosePeer(Vec<Peers>),
}

#[derive(Debug)]
pub struct TrackerResponse {
    interval: i64,
    complete: i64,
    incomplete: i64,
    peer_list: PeerModel,
}

pub fn announce(
    torrent_data: Torrent,
    info: &BDict,
    uploaded: i64,
    downloaded: i64,
    left: i64,
    compact: Option<bool>,
    event: Option<&str>,
) -> TrackerResponse {
    let mut announce_url = torrent_data.get_announce();
    let info_hash = utility::hash(info.encode());

    let query = create_tracker_query(info_hash, uploaded, downloaded, left, compact, event);

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
    compact: Option<bool>,
    event: Option<&str>,
) -> String {
    let mut PEER_ID = PeerId::new();
    let mut query = form_urlencoded::Serializer::new(String::new());
    query
        .append_pair("peer_id", &PEER_ID.value())
        .append_pair("port", &PORT.to_string())
        .append_pair("uploaded", &uploaded.to_string())
        .append_pair("downloaded", &downloaded.to_string())
        .append_pair("left", &left.to_string());

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

    match compact {
        Some(value) => {
            if value {
                query.append_pair("compact", "1");
            } else {
                query.append_pair("compact", "0");
            }
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

    let compact = match tracker_response.get::<BString>("peers") {
        Some(_) => true,
        None => false,
    };

    if compact {
        let compact_peer = tracker_response.get::<BString>("peers").unwrap().to_vec();
        TrackerResponse {
            interval,
            complete,
            incomplete,
            peer_list: PeerModel::CompactPeer(compact_peer),
        }
    } else {
        let peers_list = tracker_response.get::<BList>("peers").unwrap();
        let mut peer_list = Vec::new();
        for peer in peers_list.get() {
            let peer = peer.as_any().downcast_ref::<BDict>().unwrap();
            let peer_id = peer.get::<BString>("peer id").unwrap().to_vec();
            let ip = peer.get::<BString>("ip").unwrap().into_string().unwrap();
            let port = peer.get::<BInt>("port").unwrap().into_int();

            peer_list.push(Peers { peer_id, ip, port });
        }

        TrackerResponse {
            interval,
            complete,
            incomplete,
            peer_list: PeerModel::VerbosePeer(peer_list),
        }
    }
}
