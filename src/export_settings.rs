use std::io::{Read, ErrorKind, Write};
use std::path::Path;
use std::fs::{File, create_dir_all};
use std::collections::HashSet;

// manage content of `dir/.git/gittorrentd-daemon-export`
static SETTINGS_DIR: &str = ".gtr";
static SETTINGS_FILE: &str = "gittorrentd-daemon-export";

/// Add branches to be shared via gittorrent
///
/// The first parameter is the git repo directory. The second parameter is the list of branches to be added.
/// It adds branches resolving duplication, stores them .gtr/gittorrent-daemon-export.
pub fn include(dir: &str, new_branches: &Vec<&String>) {
    let old_branches = read_old_branches(dir);
    let old_branches: HashSet<&String> = old_branches.iter().collect();
    let new_branches: HashSet<&String> = new_branches.iter().map(|s| *s).collect();
    let final_branches: Vec<&String> = old_branches.union(&new_branches).into_iter().map(|b| *b).collect();
    write_new_branches(dir, &final_branches);
}

/// Removes branches to be shared via gittorrent
///
/// The first parameter is the git repo directory. The second parameter is the list of branches not to be shared.
/// It removes branches resolving duplication, stores new settings in .gtr/gittorrent-daemon-export.
pub fn remove(dir: &str, new_branches: &Vec<&String>) {
    let old_branches = read_old_branches(dir);
    let old_branches: HashSet<&String> = old_branches.iter().collect();
    let new_branches: HashSet<&String> = new_branches.iter().map(|s| *s).collect();
    let final_branches: Vec<&String> = old_branches.difference(&new_branches).into_iter().map(|b| *b).collect();
    write_new_branches(dir, &final_branches);
}

/// Lists branches currently shared via gittorrent
///
/// The parameter is the git repo directory. It reads branches stored in .gtr/gittorrentd-daemon-export
pub fn list(dir: &str) {
    let settings = read_old_branches(dir);
    println!("list: {settings:?}");
}

fn read_old_branches(dir: &str) -> Vec<String> {
    let settings_dir = Path::new(dir).join(SETTINGS_DIR);
    let settings_path = settings_dir.join(SETTINGS_FILE);

    match File::open(&settings_path) {
        Ok(mut file) => {
            let mut data = String::new();
            file.read_to_string(&mut data).expect("Can not read file content");
            return data.split("\n").into_iter().map(|s| String::from(s)).filter(|s| !s.eq("")).collect();
        },
        Err(e) => {
            match e.kind() {
                ErrorKind::NotFound => {
                    create_dir_all(&settings_dir).expect("Can not create gtr directory");
                    File::create(&settings_path).expect("Can not create settings file");
                    return vec!(String::from(""))
                },
                _ => panic!("Unrecognized error {e}")
            }
        }
    };
}

fn write_new_branches(dir: &str, branches: &Vec<&String>) {
    let mut sorted = branches.to_vec();
    sorted.sort();
    let stred: Vec<&str> = sorted.iter().map(|b| b.as_str()).collect();

    let settings_path = Path::new(dir).join(SETTINGS_DIR).join(SETTINGS_FILE);
    match File::create(&settings_path) {
        Ok(mut file) => file.write_all(stred.join("\n").as_bytes()).unwrap(),
        Err(e) => panic!("Cant store settings to file {e}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_entries_to_settings() {
        let dir = ".";
        let mut branches: Vec<String> = vec!["testA", "testB"].iter().map(|s| String::from(*s)).collect();
        branches.sort();
        let mut more_branches: Vec<String> = vec!["testC", "testB", "testD"].iter().map(|s| String::from(*s)).collect();
        more_branches.sort();

        let mut res_branches: Vec<String> = vec!["testA", "testB", "testC", "testD"].iter().map(|s| String::from(*s)).collect();
        res_branches.sort();

        let input_branches: Vec<&String> = branches.iter().collect();
        include(dir, &input_branches);
        let res = read_old_branches(dir);
        assert!(res.eq(&branches));

        let input_branches: Vec<&String> = more_branches.iter().collect();
        include(dir, &input_branches);
        assert!(read_old_branches(dir).eq(&res_branches));

        let input_branches: Vec<&String> = res_branches.iter().collect();
        remove(dir, &input_branches);
        assert!(read_old_branches(dir).join("").eq(""));
    }
}
