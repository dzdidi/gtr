# GTR

This is am attempt to decentralize the service you are reading this at.

# Alternatives and my problems with them:
- https://scuttlebot.io/apis/community/git-ssb.html (uses scuttelbut for communication and commits things like PRs, etc to repo itself)
- https://radicle.xyz/ - (Involved into shitcoinery)

Use torrent/dht as a transport layer for git. Here is the implementation https://github.com/cjb/GitTorrent which I am taking as an inspiration.

NOTE: While torrent has privacy issues with peers downloading data directly from each other, which means that seeders know who exactly downloads data it is worth to consider alternative solutions like GNUnet. This requires further investigation. For the sake of simplicity, while decision on technology/protocol is not made yet, future reference to torrent are as to abstract p2p data transfer protocol.

The particular implementation consists of the following parts:
  - `gti` - a CLI tool;
  - `gtd` - a daemon. It creates server (one for each local repo?) which handles requests routed by dht to send necessary pack files;
  - `git-remote-default` - a transport that handles communication via https/ssl accessed by git's remote commands like (`clone`, `fetch`, `pull` etc);
  - `git-remote-torrent` - a transport that handles communication via dht accessed by git's remote commands like (`clone`, `fetch`, `pull` etc);
  - `git-remote-holepunch` - a transport that handles communication via holepunch accessed by git's remote commands like (`clone`, `fetch`, `pull` etc);
  - `git-remote-ssb` - a transport that handles communication via scuttlebutt accessed by git's remote commands like (`clone`, `fetch`, `pull` etc);
  - `git-remote-gnunet` - a transport that handles communication via gnunet accessed by git's remote commands like (`clone`, `fetch`, `pull` etc);
  - `git-remote-nostr` - a transport that handles communication via nostr accessed by git's remote commands like (`clone`, `fetch`, `pull` etc);
  - `git-interface`:
    - generates pack files (either upon request from `gtd` or upon each new commit by git)
    - lists available branches and their corresponding references
  - `exporter` - a module responsible for:
    - persistent settings of repository/branch to share
    - for making sure that daemon is always up and running while inheriting correct settings

Use some decentralized messaging protocols for implementation of things like PRs, Issues, Reviews etc. Currently I consider:
  - nostr https://github.com/nostr-protocol/nostr
  - slashtags https://github.com/synonymdev/slashtags
  - Holepunch https://holepunch.to/
  - torrent
  - ...

# UX
The goal is to keep UX as close to plain `git` is possible. There is one nuance however. This might require a trade off where there will be one DHT server instance running per each repo.

To clone a branch use a command `git clone torrent://<hex sha1>/reponame` - where `sha1` - is a SHA1 of mutable key on DHT.
This can be converted to `git clone torrent://<user>/reponame` with `sha1` being mapped to `username`either via centralized name registry or local address book via concept of petnames

Alternatively it might make sense to forward all commands straight to git while intercepting few of them for execution of the necessary logic and passing them further.

## User flow
Note that there is no "remote repository" as such, the data is local-first.
Each branch gets announced. The `HEAD`/`master` branches are announced by default and each new branch gets added separately.

1. Create git repository if it does not exists yet and perform usual development flow;
2. Instead of pushing to remote, announce branch. This will make it available on torrent network.
There are two non-mutually-exclusive flows with torrent protocol:
  - `announce` - the source is hosted on the local machine while is available via torrent (e.g. key is pushed to DHT with value staying locally). This will require a server listening locally for connection requests, this will allow counterparties to lookup necessary branch/repo on DHT with consequent connection to local server for downloading;
  - `put` - the source is stored on DHT (e.g. both key and value stored in distributed manner over the network).

## Usage:

#### Usage: `gtr <COMMAND>`

#### Commands:
 -  `init`    create settings file and include "master" branch for sharing
 -  `share`   create settings file if not exists and share branch
 -  `list`    list currently shared branches
 -  `remove`  stop sharing given branch
 -  `pack`    ONLY FOR TESTING generate pack files
 -  `setup`   ONLY FOR TESTING setup gtr
 -  `help`    Print this message or the help of the given subcommand(s)

#### Options:
 -  `-h`, `--help`     Print help information
 -  `-V`, `--version`  Print version information

### Client mode (`git-remote-(torrent/holepunch/ssb/gnunet)`)
- `git pull` is actually doing `get` branch from DHT

### Server mode (`gtd`)
- `git push` is actually doing `announce`/`put` branch to DHT

# TODO: features configurable at build

Pluggable git transports with:
- [ ] https/ssl
- [ ] torrent
- [ ] holepunch (hyperswarn)
- [ ] scuttlebutt
- [ ] GNUnet
- [ ] nostr

Pluggable application level communication
- [ ] torrent
- [ ] holepunch (hypercore / hyperbee)
- [ ] scuttlebutt
- [ ] GNUnet
- [ ] nostr


# Data distribution approach

The goal is to provide high level of code replication across the distributed network while maintaining high level of security for its participants. 
Traditional git's approach with client-server architecture where server is responsible for maintaining access permissions for clients, even with shell wrappers may not be the most suitable.
The alternative I am considering is where each participant provides "public" read-only access for the rest of the network. Updates distribution is implemented via updating local repositories of every participants at their will. Notifications/Proposals about implementations of code changes will be handled through higher level decentralized messaging protocols, maybe with previews of diffs, which will later forward instructions for fetching/pulling data to transport protocol.
