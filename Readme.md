# GTR

This is am attempt to decentralize the service you are reading this at.

# Alternatives and my problems with them:
- https://scuttlebot.io/apis/community/git-ssb.html (uses scuttelbut for communication and commits things like PRs, etc to repo itself)
- https://radicle.xyz/ - (Involved into shitcoinery)

Use torrent/dht as a transport layer for git. Here is the implementation https://github.com/cjb/GitTorrent which I am taking as an inspiration.
NOTE: While torrent has privacy issues with peers downloading data directly from each other, which means that seeders know who exactly downloads data it is worth to consider alternative solutions like GNUnet. This requires further investigation. For the sake of simplicity, while decision on technology/protocol is not made it, future reference to torrent are as to abstract p2p data transfer protocol.
The particular implementation consists of the following parts:
  - `gti` - a CLI tool;
  - `gtd` - a daemon. It creates server (one for each local repo?) which handles requests routed by dht to send necessary pack files;
  - `git-remote-gtr` - a transport that handles communication with dht accessed by git's remote commands like (`clone`, `fetch`, `pull` etc);
  - `git-interface`:
    - generates pack files (either upon request from `gtd` or upon each new commit by git)
    - lists available branches and their corresponding references
  - `exporter` - a module reposinsible for:
    - persistent settings of repository/branch to share
    - for making sure that daemon is always up and running while inheriting correct settings

Use some decentralized messaging protocols for implementation of things like PRs, Issues, Reviews etc. Currently I consider:
  - nostr https://github.com/nostr-protocol/nostr
  - slashtags https://github.com/synonymdev/slashtags
  - torrent
  - ...

# UX
The goal is to keep UX as close to plain git is possible. There is one nuance however. This might require a trade off where there will be one DHT server instance running per each repo.

To clone a branch use a command `git clone gtr://<hex sha1>/reponame` - where `sha1` - is a sha1 of mutable key on DHT. This can be converted to `git clone gtr://<user>/reponame` with `sha1` being mapped to `username` on decentralized messaging protocol like nostr or slashtags

Alternatively it might make sense to forward all commands straight to git intercepting few of them, executing necessary logic and passing them further.

## User flow
XXX: note that there is no "repository" as such. Each branch gets announced. The `HEAD`/`master` branches are announced by default and each new branch gets added separately.

1. Create git repository if it does not yet exists and perform usual development flow;
2. Instead of pushing to remote, announce branch. This will make it available on torrent network. There are two possible flows with torrent protocol:
  - `announce` - the source is hosted on the local machine while is available via torrent (e.g. key is pushed to DHT with value staying locally). This will require a server listening locally for connection requests, this will allow counterparties to lookup necessary branch/repo on DHT with consequent connection to local server for downloading;
  - `put` - the source is stored on DHT (e.g. both key and value stored in distributed manner over the network).

### Client mode (`git-remote-gtr`)
- no `git push` but `announce`/`put` branch to DHT instead
- no `git pull` but `get` branch from DHT instead

### Server mode (`gtd`)
- no `push` but `announce`/`put` branch to DHT instead

# TODO: features configurable at build

Pluggable git transports with:
- [ ] torrent
- [ ] holepunch (hyperswarn)
- [ ] scuttlebutt
- [ ] GNUnet

Pluggable application level communication
- [ ] torrent
- [ ] holepunch (hypercore / hyperbee)
- [ ] scuttlebutt
- [ ] GNUnet
- [ ] nostr
