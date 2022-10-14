use std::io::{Read, ErrorKind, Write};
use std::ops::Deref;
use std::path::Path;
use std::fs::File;
use std::collections::HashSet;

// manage content of `dir/.git/gittorrentd-daemon-export`
static SETTINGS_DIR: &str = ".gtr";
static SETTINGS_FILE: &str = "gittorrentd-daemon-export";

/// Add branches to be shared via gittorrent
///
/// The first parameter is the git repo directory. The second parameter is the list of branches to be added.
/// It adds branches resolving duplication, stores them .gtr/gittorrent-daemon-export.
pub fn add(dir: &str, new_branches: &Vec<&str>) {
    let old_branches = read_old_branches(dir);
    let old_branches: HashSet<&str> = old_branches.iter().map(|s| s.as_ref()).collect();
    let new_branches: HashSet<&str> = new_branches.iter().map(|s| s.deref()).collect();
    let final_branches: Vec<&str> = old_branches.union(&new_branches).into_iter().map(|b| b.deref()).collect();
    write_new_branches(dir, final_branches);
}

/// Removes branches to be shared via gittorrent
///
/// The first parameter is the git repo directory. The second parameter is the list of branches not to be shared.
/// It removes branches resolving duplication, stores new settings in .gtr/gittorrent-daemon-export.
pub fn remove(dir: &str, new_branches: &Vec<&str>) {
    let old_branches = read_old_branches(dir);
    let old_branches: HashSet<&str> = old_branches.iter().map(|s| s.as_ref()).collect();
    let new_branches: HashSet<&str> = new_branches.iter().map(|s| s.deref()).collect();
    let final_branches: Vec<&str> = old_branches.difference(&new_branches).into_iter().map(|b| b.deref()).collect();
    write_new_branches(dir, final_branches);
}

/// Lists branches currently shared via gittorrent
///
/// The parameter is the git repo directory. It reads branches stored in .gtr/gittorrentd-daemon-export
pub fn list(dir: &str) {
    let settings = read_old_branches(dir);
    println!("list: {settings:?}");
}

fn read_old_branches(dir: &str) -> Vec<String> {
    let settings_path = Path::new(dir).join(SETTINGS_DIR).join(SETTINGS_FILE);
    match File::open(&settings_path) {
        Ok(mut file) => {
            let mut data = String::new();
            file.read_to_string(&mut data).expect("Can not read file content");
            return data.split("\n").into_iter().map(|s| String::from(s)).filter(|s| !s.eq("")).collect();
        },
        Err(e) => {
            match e.kind() {
                ErrorKind::NotFound => {
                    File::create(&settings_path).expect("Can not create settings file");
                    return vec!(String::from(""))
                },
                _ => panic!("Unrecognized error {e}")
            }
        }
    };
}

fn write_new_branches(dir: &str, branches: Vec<&str>) {
    let mut sorted = branches.to_vec();
    sorted.sort();

    let settings_path = Path::new(dir).join(SETTINGS_DIR).join(SETTINGS_FILE);
    match File::create(&settings_path) {
        Ok(mut file) => file.write_all(sorted.join("\n").as_bytes()).unwrap(),
        Err(e) => panic!("Cant store settings to file {e}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_entries_to_settings() {
        let dir = ".";
        let mut branches = vec!["testA", "testB"];
        branches.sort();
        let mut more_branches = vec!["testC", "testB", "testD"];
        more_branches.sort();

        let mut res_branches = vec!["testA", "testB", "testC", "testD"];
        res_branches.sort();

        add(dir, &branches);
        assert!(read_old_branches(dir).eq(&branches));

        add(dir, &more_branches);
        assert!(read_old_branches(dir).eq(&res_branches));

        remove(dir, &res_branches);
        assert!(read_old_branches(dir).join("").eq(""));
    }
}
