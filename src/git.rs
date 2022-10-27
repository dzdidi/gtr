use std::collections::{HashMap, HashSet};
use std::io::{Read, ErrorKind, Write};
use std::process::{Command, Child, Stdio, ChildStdin, ChildStdout};
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
pub fn upload_pack(dir: &str, want: &str, have: Option<&str>) {
    let git_dir = Path::new(dir).join(".git");

    let mut pack_upload = Command::new("git-upload-pack")
        .arg("--strict")
        .arg(git_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to initialize git pack upload");

    // NOTE: reference discovery
    let p = pack_upload.wait_with_output().expect("Failed to get pack upload output");
    let res = String::from_utf8(p.stdout).unwrap();
    res.split("\n").into_iter().for_each(|l| {
        println!("S: {l}")
    });


    // TODO: pack file negotiation
    let mut i_s = pack_upload.stdin.take().expect("Failed to get pack upload input stream");
    write_message(want, have, &mut i_s);

    // TODO: read from server again for ACK and NACK and for PACKFILE stream
}

fn exchange_have_want(pack_upload: &mut Child, want: &str, have: Option<&str>) {
    // XXX something to do with ACK and NACK

    // let mut o_s = pack_upload.stdout.take().expect("Failed to get pack upload output stream");
    // let p = pack_upload.wait_with_output().expect("Failed to get pack upload output");
    // let res = String::from_utf8(p.stdout).unwrap();
    // println!("READ RES: {res}");
    // read_message(&mut o_s);
}

fn read_message(o_s: &mut ChildStdout) {
    let mut message = String::new();
    o_s.read_to_string(&mut message).expect("Failed to read message");
    message.split("\n").into_iter().for_each(|l| {
        //let size = usize::from_str_radix(&l[0..4], 16).unwrap();
        let line = &l[4..l.len()];
        // XXX should we wait for input to actually appear?
    });
}
fn write_message(want: &str, have: Option<&str>, i_s: &mut ChildStdin) {
    write_pack_line(&format!("want {}", want), i_s);
    write_pack_line("", i_s);
    match have {
        Some(have) => {
            write_pack_line(&format!("have {}", have), i_s);
            write_pack_line("", i_s);
        },
        None => {}
    }
    write_pack_line("done", i_s);
}

fn write_pack_line(line: &str, s: &mut ChildStdin) {
    let res = if "".eq(line) {
        print!("C: 0000\n");
        s.write_all(b"0000\n")
    } else {
        let message = format!("{0:04}{1}\n", line.len() + 4 + 1, line);
        print!("C: {message}");
        s.write_all(message.as_bytes())
    };

    res.expect("Failed to write to pack upload input stream")
}

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

