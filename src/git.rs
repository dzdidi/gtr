use std::collections::{HashMap, HashSet};
use std::io::{BufReader, Read, ErrorKind, Write};
use std::process::{Command, Child, Stdio, ChildStdin, ChildStdout};
use std::str;
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};
use regex::Regex;

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

/// Generates necessary pack files
pub fn upload_pack(dir: &str, want: &'static str, have: Option<&'static str>) {
    let pack_upload = start_pack_upload_process(dir);

    let mut stdin = pack_upload.stdin.unwrap();
    let stdout = pack_upload.stdout.unwrap();
    let mut buffer = BufReader::new(stdout);

    request_pack_file(&mut buffer, &mut stdin, want, have);
    write_pack_file(dir, want, &mut buffer);
}

/// Store pack file to fs
fn write_pack_file(dir: &str, want:  &'static str, buffer: &mut BufReader<ChildStdout>) {
    let mut pack_content = Vec::new();
    match buffer.read_to_end(&mut pack_content) {
        Err(e) => println!("Error reading pack file content: {:?}", e),
        Ok(_) => {
            let file_path = Path::new(dir).join("..").join(format!("{want}.pack"));
            let mut file = File::create(file_path).unwrap();
            file.write_all(&pack_content).unwrap();
        }
    };
}

/// Talk to git-upload-pack until it is ready to send pack files
// NOTE: https://github.com/git/git/blob/b594c975c7e865be23477989d7f36157ad437dc7/Documentation/technical/pack-protocol.txt#L346-L393
fn request_pack_file(buffer: &mut BufReader<ChildStdout>, stdin: &mut ChildStdin, want: &'static str, have: Option<&'static str>) {
    let mut expect_nack = false;
    loop {
        // FIXME: two bytes big endian specidies message length, each message except zero messages
        // (0000) ends with new line
        // parsing should be similar to the one in lightning messages
        let mut message_buff = [0; 65535]; // FFFF
        match buffer.read(&mut message_buff) {
            Err(e) => println!("Error requesting pack file: {:?}", e),
            Ok(_) => {
                let line = read_line(String::from_utf8(message_buff.to_vec()).unwrap());

                let end_of_list = line.contains("\n0000");
                // We do not need to check git server refs as we know them from ls
                if !(expect_nack || end_of_list) { continue; }

                if end_of_list {
                    write_message(want, have, stdin);
                    expect_nack = true;
                    continue;
                }

                match have {
                    Some(_) => ack_objects_continue(&line) && return,
                    None => wait_for_nak(&line) && return,
                };
            }
        };
    }
}

/// Start git-upload-pack server
fn start_pack_upload_process(dir: &str) -> Child {
    let git_dir = Path::new(dir).join(".git");
    let pack_upload = Command::new("git-upload-pack")
        .arg("--strict")
        .arg(git_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to initialize git pack upload");

    return pack_upload
}

/// Identify git pack server nack response
fn wait_for_nak(line: &str) -> bool {
    return !line.eq("NAK")
}

/// Identify git pack server ack response
fn ack_objects_continue(line: &str) -> bool {
    let ack_regex = Regex::new("^ACK").unwrap();
    let is_ack = ack_regex.is_match(line);
    let con_regex = Regex::new("continue$").unwrap();
    let is_con = con_regex.is_match(line);

    return is_ack && !is_con
}

/// Read git pack server response
fn read_line(line: String) -> String {
    // NOTE lines size is actually passed
    // let size = usize::from_str_radix(&line[0..4], 16).unwrap();
    let line = String::from(&line[4..line.chars().count()]);
    return line
}

/// Complete message sent to server for packfile negotiation
fn write_message(want: &str, have: Option<&str>, stdin: &mut ChildStdin) {
    write_pack_line(&format!("want {}", want), stdin);
    write_pack_line("", stdin);
    match have {
        Some(have) => {
            write_pack_line(&format!("have {}", have), stdin);
            write_pack_line("", stdin);
        },
        None => {}
    }
    write_pack_line("done", stdin);
}

/// Write line to stdin for git pack communication
fn write_pack_line(line: &str, stdin: &mut ChildStdin) {
    if "".eq(line) {
        stdin.write_all(String::from("0000").as_bytes()).unwrap()
    } else {
        let message = format!("{0:04x}{1}\n", line.as_bytes().len() + 4 + 1, line);
        stdin.write_all(message.as_bytes()).unwrap();
    }
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

/// Add gtr related files to gitignore
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

