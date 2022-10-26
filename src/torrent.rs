// TODO:
// 1. spin up DHT and make it listen for connections on on configurable <listen net port>
// NOTE: bip-dht will immediately try to bootstrap so with provide bootstrap nodes
// NOTE: for local setup bip-dht needs three nodes, as table will not be populated
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
