use std::{fs, path::Path};

fn main() {
    // start DHT, sync
    // on ready
    // TODO:

    let sharing_flag = "gittorrent-daemon-export-ok";
    let git_path = Path::new(".git");

    if !git_path.join(sharing_flag).exists() {
        return
    }

    let folder = fs::read_dir(git_path).unwrap();
    for path in folder {
        let file_name = path.unwrap().file_name();

        if file_name == sharing_flag {
            continue;
        }
        // XXX share only head
        if file_name == "HEAD" {
            println!("Name: {:?}", file_name);
        }
    }
}
