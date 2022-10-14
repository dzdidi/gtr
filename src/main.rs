use std::env;

use gtr::git::{is_git, ignore_gtr, ls_remote};
use gtr::export_settings::{add, remove, list};

// XXX UX:
// Original gittorrent allows user to share all/many dirs from common parent directory by
// running gittorrentd in it. All leaves that have `.gtr/gittorrent-daemon-export-ok` file
// will be shared but only their master and head.
//
// My approach so far is to share repo by running gittorrentd in it and providing branches
// to share as arguments, with master (and HEAD?) being defaults. The list of provided
// branches will be stored in `.gtr/gittorrentd-daemon-export` file
//
// XXX: rough UX on cli
// TODO:
// 1. check if ".gtr/gittorrentd-daemon-export" exists and readable and writable
// Read from it for default branches to share
// 2. check if command line argument was passed (space separated) or use master as a default
// append them to ".gtr/gittorrentd-daemon-export" (space or newline separated)

fn main() {
    // TODO:
    // start DHT, sync
    // on ready

    let args = Vec::from_iter(env::args());
    let mut args: Vec<&str> = args.iter().map(|v| v.as_ref()).collect();

    args.remove(0); // first argument is a command name
    let dir = args.remove(0);

    // XXX can be combined in a single `setup` call;
    is_git(dir);
    ignore_gtr(dir);

    let refs = ls_remote(dir);
    println!("{refs:?}");

    // XXX pass refs to make sure that corresponding branches exist???
    match args.remove(0) {
        "add" => add(dir, &args),
        "remove" => remove(dir, &args),
        "list" => list(dir),
        _ => panic!("Unrecognized command")
    }
}
