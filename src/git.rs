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

// XXX wait for server to send 0000 and start sending wants
// for some reason I see no 0000 in servers response
pub fn upload_pack(dir: &str, want: &'static str, have: Option<&'static str>) {
    let pack_upload = start_pack_upload_process(dir);

    let mut stdin = pack_upload.stdin.unwrap();
    let stdout = pack_upload.stdout.unwrap();
    let mut buffer = BufReader::new(stdout);

    let mut expect_nack = false;
    let mut expect_pack = false;
    loop {
        if expect_pack {
            write_pack_file(dir, want, &mut buffer);
            break
        } else {
            // NOTE: we already know all the refs thus do not need to validate anything
            // XXX Vec::new() - fails out of bound
            let mut message_buff = [0; 65535]; // FFFF
            match buffer.read(&mut message_buff) {
                Err(e) => println!("Error generating pack file: {:?}", e),
                Ok(_) => {
                    let line = String::from_utf8(message_buff.to_vec()).unwrap();
                    let line = read_line(line);
                    println!("S: {line}");

                    if expect_nack {
                        let res = match have {
                            Some(_) => ack_objects_continue(&line),
                            None => wait_for_nak(&line)
                        };
                        if res {
                            expect_pack = true;
                        }
                    } else {
                        if line.contains("\n0000") {
                            write_message(want, have, &mut stdin);
                            expect_nack = true;
                        }
                    };
                    continue;
                }
            };
        }
    };
}

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

// fn request_pack_file(buffer: &mut BufReader<ChildStdout>, want: &'static str, have: &'static str) { }

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

/// Generates necessary pack files
// NOTE: https://github.com/git/git/blob/b594c975c7e865be23477989d7f36157ad437dc7/Documentation/technical/pack-protocol.txt#L346-L393

fn wait_for_nak(line: &str) -> bool {
    return !line.eq("NAK")
}

fn ack_objects_continue(line: &str) -> bool {
    let ack_regex = Regex::new("^ACK").unwrap();
    let is_ack = ack_regex.is_match(line);
    let con_regex = Regex::new("continue$").unwrap();
    let is_con = con_regex.is_match(line);

    return is_ack && !is_con
}

fn read_line(line: String) -> String {
    // NOTE lines size is actually passed but 
    // let size = usize::from_str_radix(&line[0..4], 16).unwrap();
    let line = String::from(&line[4..line.chars().count()]);
    return line
}

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

fn write_pack_line(line: &str, stdin: &mut ChildStdin) {
    if "".eq(line) {
        println!("\nC: 0000");
        stdin.write_all(String::from("0000").as_bytes()).unwrap()
    } else {
        let message = format!("{0:04x}{1}\n", line.as_bytes().len() + 4 + 1, line);
        println!("C: {message}");
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

