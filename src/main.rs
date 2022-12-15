// use std::env;
use std::path::Path;

use clap::Parser;

use gtr::git::{gtr_setup, select_exsiting_branches, upload_pack};
// use gtr::export_settings::{include, remove, list};

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


// TODO: see https://github.com/clap-rs/clap/blob/master/examples/git.rs
#[derive(clap::ValueEnum, Clone, Debug)]
enum Action {
    Include,
    Remove,
    List,
}
/// Arguments for gtr
#[derive(Parser, Debug)]
struct Args {
    /// directory to be handled by gtr
    // #[arg(short, long, default_value_t = String::from("."))]
    #[arg(short, long)]
    dir: String,
    /// action to be made by gtr
    #[arg(short, long, value_enum)]
    action: Action,
    #[arg(short, long, value_enum)]
    branches: String[],
}
fn main() {
    let arguments = Args::parse();
    println!("{:?}", arguments);
    gtr_setup(&arguments.dir);

//    if args.len() > 0 {
//        let action = args.remove(0).as_str(); // third is action
//        match action {
//            // NOTE: alternatively forward commands to the git itself
//            "include" => include(dir, &select_exsiting_branches(dir, &args).iter().collect()),
//            "remove" => remove(dir, &args),
//            "list" => list(dir), // and exit?
//            // NOTE: cli test
//            "pack" =>{
//                let want = "447990420af9fe891cfe7880d04d9769e4168f7a";
//                let have = Some("cced046c2b0435ff258de91580720427316f07ae");
//                upload_pack(dir, want, have)
//            },
//            _ => panic!("Unrecognized command"),
//        }
//    }
}

fn act(action: Action, dir: String) {
    match action {
        // NOTE: alternatively forward commands to the git itself
        Include => include(dir, &select_exsiting_branches(dir, &args).iter().collect()),
        Remove => remove(dir, &args),
        List => list(dir), // and exit?
        // NOTE: cli test
        pack =>{
            let want = "447990420af9fe891cfe7880d04d9769e4168f7a";
            let have = Some("cced046c2b0435ff258de91580720427316f07ae");
            upload_pack(dir, want, have)
        },
        _ => panic!("Unrecognized command"),
    }
}
