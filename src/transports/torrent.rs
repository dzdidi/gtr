// #[cfg(feature = "torrent")]
use std::thread;
use std::io::{self, Read};
use std::collections::HashSet;

use std::net:: SocketAddr;
use std::path::PathBuf;

use tokio::runtime::Runtime;

use bip_dht::{MainlineDht, DhtBuilder, PeerId};
use bip_util::bt::InfoHash;
use bip_handshake::{Handshaker, HandshakerBuilder, InitiateMessage, Protocol};
use bip_handshake::transports::TcpTransport;

use crate::config::config_file;

pub async fn get_dht(dir: &PathBuf) {
    let torrent_config  = get_torrent_config(dir).await;
    let handshaker = get_handshaker(&torrent_config).await;
    let dht = start_dht(&torrent_config, &handshaker);

    // Spawn a thread to listen to and report events
    let events = dht.events();
    thread::spawn(move || {
        // TODO: handle events
        for event in events {
            println!("\nReceived Dht Event {:?}", event);
        }
    });

    let hash = InfoHash::from_bytes(b"My Unique Info Hash");
    // Let the user announce or search on our info hash
    let stdin = io::stdin();
    let stdin_lock = stdin.lock();
    for byte in stdin_lock.bytes() {
        match &[byte.unwrap()] {
            b"a" => dht.search(hash.into(), true),
            b"s" => dht.search(hash.into(), false),
            _   => ()
        }
    }
}

struct TorrentConfig {
    pub bootstrap_address: SocketAddr,
    pub bind_address: SocketAddr,
    pub source: SocketAddr,
}

async fn get_torrent_config(dir: &PathBuf) -> TorrentConfig {
    let config = config_file::read_or_create(dir).await;
    let torrent_config = match config.transport.torrent {
        None => panic!("Torrent in not configured"),
        Some(torrent) => torrent
    };

    return TorrentConfig {
        bootstrap_address: SocketAddr::new(torrent_config.router.addr.parse().unwrap(), torrent_config.router.port),
        bind_address: SocketAddr::new(torrent_config.bind.addr.parse().unwrap(), torrent_config.bind.port),
        source: SocketAddr::new(torrent_config.bind.addr.parse().unwrap(), torrent_config.bind.port),
    }
}

// XXX bip_dht relies on outdated version of a bip_handshaker
//  in bit_dht Handshaker is a Trait while in bip_handshaker it is a Struct
async fn get_handshaker(torrent_config: &TorrentConfig) -> Handshaker {
    // TODO: from config?
    let u_torrent_peer_id = (*b"-UT2060-000000000000").into();

    // TODO: hash generation (this is probably immutable hash)
    let hash = InfoHash::from_bytes(b"My Unique Info Hash");

    // XXX: bit_handshaler relies on deprecated version of tokio and requires explicit
    // tokio_core::reactor::Handle as a parameter to build, while current version of tokio
    // provides Handle.
    let rt = Runtime::new().unwrap();
    let handshaker = HandshakerBuilder::new()
        .with_peer_id(u_torrent_peer_id)
        .build(TcpTransport, *rt.handle())
        .unwrap()
        .send(InitiateMessage::new(
                Protocol::BitTorrent,
                hash,
                torrent_config.bootstrap_address,
            )
        )
        .await
        .unwrap();

    return handshaker
}

fn start_dht(torrent_config: &TorrentConfig, handshaker: &HandshakerBuilder) -> MainlineDht {
    DhtBuilder::with_node(torrent_config.bootstrap_address)
        .set_source_addr(torrent_config.source)
        .set_read_only(false)
        .start_mainline(handshaker)
        .unwrap()
}

// TODO: implement transport trait so that different transports can get consistently injected
//
// TODO:
// 1. spin up DHT and make it listen for connections on on configurable <listen net port>
// NOTE: bip-dht will immediately try to bootstrap so with provided bootstrap nodes
// NOTE: for local setup bip-dht needs three nodes, as table will not be populated
// NOTE: UPDATE: router is not included into nodes table, thus if DHT is started with node instead
// of with router, two nodes should suffice
//     by information from the initial node
//
// 2. anounce "<sha>" to DHT on configurable <announce net port>
// NOTE: Data structures used in gittorrentd:
// - UserProfile{
//      repositories: {
//          "<repo name>: {
//              "<branch (ref name)>": "<sha>"
//          }
//      }
// }
// - Announced Refs:
// {
//   "<sha>": "repo name"
// }
// 3. publish mutable key
// Stringify UserProfile object, confirm its lengths is less than 950 bytes
// Sign value with generated keys (TODO: check when to do this and is nostr keys can/should be
// reused)
// Create opts object:
// {
//      k: 32bytes padded public key
//      seq: sequnce number starting from 0 (NOTE: gittorrentd does not increment it and it does
//          not seem to be needed)
//      v: buffered serialized Repository object
//      sig: Concatenation of 32bytes padded ed25519 signature r and s values (standard)
// }
//
// 4. Create a net server and with web socket on <net announce port> and ut wire for torrent and metadata
// NOTE: gittorrentd uses ut_gittorrent and ut_metadata. Their role might be partially or fully
//      implemented in bip-dht, bip-metainfo and/or bip-utracker
// Pipe server's socket into wire
// Wire listens to two events: "handshake" and "generatePack"
// 4.1 Handshake (gets InfoHash, peerId)
//      generate our peerId (NOTE: should be done by bip-dht)
//      respond to handshake with received infoHash and our peerId
// 4.2 generatePack (gets sha)
//      if sha is not part of announcedRefs -> exit
//      get repo name by sha from Announced Refs
//      do git upload pack 
//          in gittorrend it is implemented as a call to git-upload-pack
//          it is part of git's pack protocol
//          TODO: investigate
//      when pack is created store it as <sha>.pack file
//      create dht with disabled tracking
//      seed pack file to dht by sending torrent
