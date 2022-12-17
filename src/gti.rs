use clap::{arg, Command, Parser};
use clap::Parser;

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
    branches: String,
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

pub fn cli() -> Command {
    let matches = command!()
        .subcommand_required(true) // can't just run gtr?
        .arg_required_else_help(true)
        .allow_external_subcommands(true) // wtf is this?
        .subcommand(
            Command::new("include")
            .about("add new branches to gtr and start sharing it via dht")
            .arg(arg!(dir: [PATH]))
        ).subcommand(
            Command::new("remove")
            .about("remove new project to gtr and start sharing it via dht")
            .arg(arg!(dir: [PATH]))
        ).subcommand(
            Command::new("list")
            .about("add new project to gtr and start sharing it via dht")
            .arg(arg!(dir: [PATH]))
        ).subcommand(
            Command::new("pack")
            .about("add new project to gtr and start sharing it via dht")
            .arg(arg!(dir: [PATH]))
        ); // for test?
}
