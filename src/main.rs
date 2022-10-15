use std::env;

use gtr::git::{setup, select_exsiting_branches};
use gtr::export_settings::{add, remove, list};

// XXX UX:
// Original gittorrent allows user to share all/many dirs from common parent directory by
// running gittorrentd in it. All leaves that have `.gtr/gittorrent-daemon-export-ok` file
// will be shared but only their master and head.
//
// My approach so far is to share repo by running gittorrentd in it and providing branches
// to share as arguments, with master (and HEAD?) being defaults. The list of provided
// branches will be stored in `.gtr/gittorrentd-daemon-export` file
fn main() {
    // TODO:
    // start DHT, sync
    // on ready

    let args = Vec::from_iter(env::args());
    let mut args: Vec<&String> = args.iter().collect();

    args.remove(0); // first argument is a command name
    let dir = args.remove(0);
    let action = args.remove(0).as_str();

    setup(dir);

    let existing_branches = select_exsiting_branches(dir, &args);
    let existing_branches: Vec<&String> = existing_branches.iter().collect();

    match action {
        "add" => add(dir, &existing_branches),
        "remove" => remove(dir, &args),
        "list" => list(dir),
        _ => panic!("Unrecognized command")
    }

    // TODO: read branches
    // select the current branches from file
    // get their hash from git
    //
}
