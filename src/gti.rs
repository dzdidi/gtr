use clap::{arg, command, Command, value_parser};
use std::path::PathBuf;

// fn act(action: Action, dir: String) {
//     match action {
//         // NOTE: alternatively forward commands to the git itself
//         Include => include(dir, &select_exsiting_branches(dir, &args).iter().collect()),
//         Remove => remove(dir, &args),
//         List => list(dir), // and exit?
//         // NOTE: cli test
//         pack =>{
//             let want = "447990420af9fe891cfe7880d04d9769e4168f7a";
//             let have = Some("cced046c2b0435ff258de91580720427316f07ae");
//             upload_pack(dir, want, have)
//         },
//         _ => panic!("Unrecognized command"),
//     }
// }

pub fn cli() -> Command {
    let init = Command::new("init")
        .about("create settings file and include master branch for sharing")
        .arg(arg!(path: [PATH])
            .value_parser(value_parser!(PathBuf))
            .default_value("."),
        );

    let share = Command::new("share")
        .about("create settings file if not exists and share branch")
        .arg(arg!(path: [PATH])
            .value_parser(value_parser!(PathBuf))
            .default_value("."),
        )
        .arg(arg!(branch: [BRANCH]).required(true));

    let list = Command::new("list")
        .about("list currently shared branches")
        .arg(arg!(path: [PATH])
            .value_parser(value_parser!(PathBuf))
            .default_value("."),
        );

    let remove = Command::new("remove")
        .about("stop sharing given branch")
        .arg(arg!(path: [PATH])
            .value_parser(value_parser!(PathBuf))
            .default_value("."),
        )
        .arg(arg!(branch: [BRANCH]).required(true));

    let _pack = Command::new("pack")
        .about("ONLY FOR TESTING generate pack files")
        .arg(arg!(path: [PATH])
            .value_parser(value_parser!(PathBuf))
            .default_value("."),
        )
        .arg(arg!(want: [WANT]))
        .arg(arg!(have: [HAVE]));

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
