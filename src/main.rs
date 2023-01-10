// use std::env;
use gtr::git_interface::upload_pack;
use gtr::config::branches::{include, remove, list};
// TODO: use a feature and inject in a different place
use gtr::transports::torrent::get_dht;
use gtr::gti::cli;

// XXX UX:
// Original gittorrent allows user to share all/many dirs from common parent directory by running gittorrentd in it.
// All leaves that have `.gtr/gittorrent-daemon-export-ok` file will be shared but only their master and head.
//
// TODO: FIXME: the approach described bellow is wrong as it will require one web server per repo!
// My approach so far is to share repo by running gittorrentd in it (or at the moment, passing dirname)
// and providing branches to share as arguments, with master (and HEAD?) being defaults. The list of provided branches will
// be stored in `.gtr/gittorrentd-daemon-export` file
//
// NOTE: UX/Arch: 
// 1. TODO: stop passing directory as a first argument, and always use current directory (for
//    testing purposes there might be a trade-off with allowing to be passed as env var or option
// 2. When user uses tool for multiple local repositories, the new DHT instance should be created
//    once but reused for all future cases. It also might make sense to disable "seeding" through
//    some persisted configurable option. DHT instance needs to be started after reboot.

// NOTE: basic CLI functionality might be completed in current implementation
//
// XXX consider split for CLI and Daemon or ALTERNATIVE: provide gtr CLI as tool which will act as a wrapper for git.
// By default it passes commands to git and lets git execute them. With few following exceptions:
// * `init` passes to git and does what current implementation of `setup` does with default branches
// * `push` adds branch to list of shared branches?
// * ...
// For private repository utilize idea of gitconfig with settings for private/public repos and for list of user's keys
// who are allowed to download it. For this bittorrent's handshake/seeding process must be tweaked.
//
// TODO: create nested "test" directory to run tests within it.
// TODO: (what is this) read branches from file? get their hash from git follow original gittorrent
// TODO: write integration test


#[tokio::main]
async fn main() {
    match cli().get_matches().subcommand() {
        Some(("init", sub_matches)) => include(
            sub_matches.get_one("path").unwrap(),
            &vec![&String::from("master")]
        ).await,
        Some(("share", sub_matches)) => {
            let branches = sub_matches
                .get_many::<String>("branches")
                .unwrap_or_default()
                .collect::<Vec<_>>();

            include(sub_matches.get_one("path").unwrap(), &branches).await;
        }
        Some(("list", sub_matches)) => {
            let branches = list(sub_matches.get_one("path").unwrap()).await;
            println!("shared branches: {branches:?}");
        },
        Some(("remove", sub_matches)) => {
            let branches = sub_matches
                .get_many::<String>("branches")
                .unwrap_or_default()
                .collect::<Vec<_>>();
            remove(sub_matches.get_one("path").unwrap(), &branches).await;
        }
        Some(("pack", sub_matches)) => {
            let want = "66ef7ea67c18d2341afb8c1521afbab31014e62f"; // refs/heads/test/want
            let have = Some("da13823f7206ed470cdab7c98285cd706ae1dcbe"); // refs/heads/test/have

            let dir = sub_matches.get_one("path").unwrap();
            upload_pack(dir, want, have).await;
        }
        _ => unreachable!(), // If all subcommands are defined above, anything else is unreachabe!()
    }
}
