use clap::{arg, Arg, ArgAction, command, Command, value_parser};
use std::path::PathBuf;

pub fn cli() -> Command {
    let path_arg = Arg::new("path")
        .short('p')
        .long("path")
        .help("Path to repository to be shared")
        .default_value(".")
        .value_parser(value_parser!(PathBuf));

    let branches_arg = Arg::new("branches")
        .short('b')
        .long("branches")
        .help("space separated list of git branches")
        .default_value("master")
        .value_delimiter(',')
        .action(ArgAction::Append);

    let init = Command::new("init")
        .about("create settings file and include master branch for sharing")
        .arg(&path_arg);

    let share = Command::new("share")
        .about("create settings file if not exists and share branch")
        .arg(&branches_arg)
        .arg(&path_arg);

    let list = Command::new("list")
        .about("list currently shared branches")
        .arg(&path_arg);

    let remove = Command::new("remove")
        .about("stop sharing given branch")
        .arg(&branches_arg)
        .arg(&path_arg);

    let _pack = Command::new("pack")
        .about("ONLY FOR TESTING generate pack files")
        .arg(arg!(want: [WANT]))
        .arg(arg!(have: [HAVE]))
        .arg(&path_arg);

    command!()
        .subcommand_required(true) // can't just run gtr?
        .arg_required_else_help(true)
        .allow_external_subcommands(true) // wtf is this?
        .subcommand(init)
        .subcommand(share)
        .subcommand(list)
        .subcommand(remove)
        .subcommand(_pack)
}
