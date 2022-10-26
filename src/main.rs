use std::env;

use gtr::git::{setup, select_exsiting_branches};
use gtr::export_settings::{add, remove, list};

// XXX UX:
// Original gittorrent allows user to share all/many dirs from common parent directory by running gittorrentd in it.
// All leaves that have `.gtr/gittorrent-daemon-export-ok` file will be shared but only their master and head.
//
//
// FIXME: the approach described bellow is wrong as it will require one web server per repo!
// My approach so far is to share repo by running gittorrentd in it and providing branches to share as arguments, with
// master (and HEAD?) being defaults. The list of provided branches will be stored in `.gtr/gittorrentd-daemon-export`
// file

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
// TODO: investigate how keys for user's identity from bittorrent and nostr can be made to work together
fn main() {
    let args = Vec::from_iter(env::args());
    let mut args: Vec<&String> = args.iter().collect();

    args.remove(0); // first argument is a command name
    let dir = args.remove(0); // second is a target directory
    setup(dir);

    if args.len() > 0 {
        let action = args.remove(0).as_str(); // third is action
        match action {
            "add" => add(dir, &select_exsiting_branches(dir, &args).iter().collect()),
            "remove" => remove(dir, &args),
            "list" => list(dir), // and exit?
            _ => panic!("Unrecognized command")
        }
    }

    // TODO:
    // read branches from file
    // get their hash from git
    // follow original gittorrent

    // TODO: write integration test
}
