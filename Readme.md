# GTR

This is am attempt to decentralize the service you are reading this at.

# Alternatives and my problems with them:
- https://scuttlebot.io/apis/community/git-ssb.html (uses scuttelbut for communication and commits things like PRs, etc to repo itself)
- https://radicle.xyz/ - (Involved into shitcoinery)

The idea described bottom up:

1. Use torrent/dht as a transport layer of git. Here is the implementation https://github.com/cjb/GitTorrent which I am taking as an inspiration. It consists of three parts:
  - `gtr` - a CLI tool for managing repository
  - `gtrd` (`gittorrentd`) - daemon which spins up dht for a given directory (repository)
  - `git-remote-gtr` - a 'bin' to manage transport over DHT
2. Use some decentralized messaging protocols for implmentation of things like PRs, Issues, Reviews etc. Currently I consider:
  - nostr https://github.com/nostr-protocol/nostr
  - slashtags https://github.com/synonymdev/slashtags
  - ...

# UX
The goal is to keep UX as close to plain git is possible. There is one nuance however. This might require a trade off where there will be one DHT server instance running per each repo.

To clone a repository use a command `git clone gtr://<hex sha1>/reponame` - where `sha1` - is a sha1 of mutable key on DHT. This can be converted to `git clone gtr://<user>/reponame` with `sha1` being mapped to `username` on decentralized messaging protocol like nostr or slashtags

Alternatively it might make sense to forward all commands straight to git intercepting few of them, executing necessary logic and passing them further.
