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
use std::collections::HashSet;
use std::net::{SocketAddr, Ipv4Addr, SocketAddrV4};
use std::thread;
use std::io::{self, Read};

use bip_dht::{DhtBuilder, Router};
use bip_handshake::Handshaker;
use bip_util::bt::{InfoHash, PeerId};
use bip_handshake::{HandshakerBuilder, InitiateMessage, Protocol};
use bip_handshake::transports::TcpTransport;

use crate::config::config_file;

pub fn start_dht(dir: &PathBuf) {
    // TODO: hash generation (this is probably immutable hash
    let hash = InfoHash::from_bytes(b"My Unique Info Hash");

    let config = config_file::read_or_create(dir).await;
    // TODO: move out
    let router = match config.transport.torrent {
        None => panic("Torrent in not configured"),
        Some(torrent) => SocketAddr::V4(
            SocketAddrV4::new(
                Ipv4Addr::new(torrent.router.addr),
                torrent.router.port
                )
            )
    };
    let bind_address = match config.transport.torrent {
        None => panic("Torrent in not configured"),
        Some(torrent) => SocketAddr::V4(
            SocketAddrV4::new(
                Ipv4Addr::new(torrent.bind.addr),
                torrent.bind.port
                )
            )
    };
    let source = SocketAddr::V4(bind_address);

    // TODO: use more sophisticated handshaker, see example from bip_handshake
    //let handshaker = SimpleHandshaker{ filter: HashSet::new(), count: 0 };
    let peer_id = (*b"-UT2060-000000000000").into(); // bootstrap peer_id???
    let handshaker = HandshakerBuilder::new();

    let dht = DhtBuilder::with_node(router)
        .set_source_addr(source)
        .set_read_only(false)
        .start_mainline(handshaker)
        .unwrap();

    // Spawn a thread to listen to and report events
    let events = dht.events();
    thread::spawn(move || {
        // TODO: handle events
        for event in events {
            println!("\nReceived Dht Event {:?}", event);
        }
    });
    
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
