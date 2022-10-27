use std::collections::{HashMap, HashSet};
use std::io::{Read, ErrorKind, Write, stdin, Stdin};
use std::process::{Command, Stdio, ChildStdin};
use std::str;
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};

static SETTINGS_DIR: &str = ".gtr";

/// Selects only existing branches
pub fn select_exsiting_branches(dir: &str, branches: &Vec<&String>) -> Vec<String> {
    let availalbe: HashSet<String> = ls_remote(dir).into_keys().collect();
    let requested: HashSet<String> = branches.iter().map(|s| String::from("refs/heads/") + s).collect();
    return availalbe.intersection(&requested).into_iter().map(|s| String::from(s)).collect()
}

/// Checks if directory is a git repository, adds service folder to gitignore
pub fn setup(dir: &str) {
    is_git(dir);
    ignore_gtr(dir);
}

/// Returns hash of Ref for each branch of given repository as well as current HEAD
pub fn ls_remote(dir: &str) -> HashMap<String, String> {
    let refs = Command::new("git").arg("ls-remote").arg(dir).output().unwrap();
    let refs = String::from_utf8(refs.stdout).unwrap();
    return refs
        .split("\n")
        .into_iter()
        .filter(|r| !String::from("\n").eq(r) && !String::from("").eq(r))
        .map(|r| {
            let s: Vec<&str> = r.split("\t").collect();
            return (String::from(s[1]), String::from(s[0]))
        })
        .collect();
}

// TODO: add function for generating git pack. See:
// https://github.com/git/git/blob/b594c975c7e865be23477989d7f36157ad437dc7/Documentation/technical/pack-protocol.txt#L346-L393
pub fn upload_pack(dir: &str, want: &'static str, have: &'static str) {
    let git_dir = Path::new(dir).join(".git");

    let mut pack_upload = Command::new("git-upload-pack")
        .arg("--strict")
        .arg(git_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to initialize git pack upload");

    let mut pack_upload_stdin = pack_upload.stdin
        .take()
        .expect("Failed to get pack upload input stream");
    std::thread::spawn(move || {
        write_pack_message(want, have, &pack_upload_stdin);
    });

    let output = pack_upload.wait_with_output().expect("Failed to read from pack upload output stream");
    let message = String::from_utf8(output.stdout).expect("Failed to parse pack upload output");

    // TODO:
    // NOTE: gittorrent reads per line so that it can pipe parsed output to stream
    // for now we just read everything together which might be problematic if output
    // is too long when there are too many packs

    println!("Finished have want dialog");
    println!("{}", message);
}

/// Creates message to be written to git-upload-pack process stdin
fn write_pack_message(want: &str, have: &str, mut s: &ChildStdin) {
    write_pack_line(&format!("want {}", want), s);
    write_pack_line("", s);
    write_pack_line(&format!("have {}", have), s);
    write_pack_line("done", s);
}

fn write_pack_line(line: &str, mut s: &ChildStdin) {
    let res = if "".eq(line) {
        s.write_all(b"0000")
    } else {
        s.write_all(format!("{0:04}{1}\n", line.len() + 4 + 1, line).as_bytes())
    };

    res.expect("Failed to write to pack upload input stream")
}
// #[cfg(test)]
// mod test {
//     use super::*;
// 
//     #[test]
//     fn writes_empty_pack_line() {
//         let message = write_pack_line(None);
//         assert!(message.eq("0000"));
//     }
// 
//     #[test]
//     fn write_non_empty_pack_line() {
//         let line = "random_line";
//         let message = write_pack_line(Some(line));
//         assert!(message.eq("0016random_line\n"));
//     }
// 
//     #[test]
//     fn creates_pack_message_without_have() {
//         let want = "wanted_sha";
//         let message = create_pack_message(want, None);
//         assert!(message.eq("want wanted_sha\ndone\n"));
//     }
// 
//     #[test]
//     fn creates_pack_message_with_have() {
//         let want = "wanted_sha";
//         let have = "haved_sha";
//         let message = create_pack_message(want, Some(have));
//         assert!(message.eq("want wanted_sha\nhave haved_sha\ndone\n"));
//     }
// }



/// Add .gtr directory to gitignore in provided repository
fn ignore_gtr(dir: &str) {
    let gitignore_path = Path::new(dir).join(".gitignore");
    match File::open(&gitignore_path) {
        Ok(mut file) => {
            let mut data = String::new();
            file.read_to_string(&mut data).expect("Can not read file content");

            let gtr_ignored = data.split("\n").into_iter().any(|s| String::from(SETTINGS_DIR).eq(s));
            if !gtr_ignored { store_in_gitignore(&gitignore_path); }
        },
        Err(e) => match e.kind() {
            ErrorKind::NotFound => store_in_gitignore(&gitignore_path),
            _ => panic!("Unrecognized error {e}")
        }
    }
}

fn store_in_gitignore(gitignore_path: &PathBuf) {
    let store = |mut file: File| { file.write_all((String::from("\n") + SETTINGS_DIR).as_bytes()).unwrap() };

    match OpenOptions::new().write(true).append(true).open(gitignore_path) {
        Ok(file) => store(file),
        Err(_) => {
            let file = File::create(&gitignore_path).unwrap();
            OpenOptions::new().write(true).append(true).open(gitignore_path).unwrap();
            store(file);
        }
    }
}
/// Panics if provided directory is not a git repository
fn is_git(dir: &str) {
    if !Path::new(dir).join(".git").exists() {
        panic!("Not a git repository");
    }
}

