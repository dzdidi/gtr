use std::collections::{HashMap, HashSet};
use std::io::{BufReader, Read, ErrorKind, Write, BufRead};
use std::process::{Command, Child, Stdio};
use std::str;
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use std::sync::{atomic::AtomicBool, atomic::Ordering, Arc, Mutex};
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
// NOTE: https://github.com/git/git/blob/b594c975c7e865be23477989d7f36157ad437dc7/Documentation/technical/pack-protocol.txt#L346-L393
pub fn upload_pack(dir: &str, want: &str, have: Option<&str>) {
    let git_dir = Path::new(dir).join(".git");

    let (sender1, receiver1) = channel(); // channel to read data from pack server
    let (sender2, receiver2) = channel(); // channel to send data to pack server

    let mut upload_pack_process = start_upload_pack_process(git_dir.to_str().unwrap(), sender1, receiver2);
    // NOTE: reference discovery
    write_message(want, have, &sender2);

    // XXX
    //start_upload_pack_command_thread(Mutex::new(sender2.clone()));

    let should_terminate = Arc::new(AtomicBool::new(false));

    while !should_terminate.load(Ordering::Relaxed) {
        match receiver1.try_recv() {
            Ok(line) => {
                // NOTE: reference discovery (cont)
                // NOTE: server lists refs, which are already known from the ls-remote command
                // so we wait for it to signal end.
                let line = read_line(line);

                // TODO: read from server again for ACK and NACK and for PACKFILE stream
                let res = match have {
                    Some(_) => ack_objects_continue(&line),
                    None => wait_for_nak(&line)
                };

                if !res {
                    return
                }
                // TODO: pack file negotiation
                continue;
            }
            Err(TryRecvError::Empty) => {
                sleep(Duration::from_secs(1));
                continue;
            }
            Err(e) => {
                println!("Error: {:?}", e);
            }
        }
    }

    upload_pack_process.kill().expect("Failed terminate pack upload process");
}

fn wait_for_nak(line: &str) -> bool {
    return !line.eq("NAK")
}

fn ack_objects_continue(line: &str) -> bool {
    let ack_regex = Regex::new("^ACK").unwrap();
    let is_ack = ack_regex.is_match(line);
    let con_regex = Regex::new("continue$").unwrap();
    let is_con = con_regex.is_match(line);

    return !is_ack || is_con
}

fn start_upload_pack_process(dir: &str, sender: Sender<String>, receiver: Receiver<String>) -> Child {
    let mut pack_upload = Command::new("git-upload-pack")
        .arg("--strict")
        .arg(dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to initialize git pack upload");

    start_upload_pack_process_thread(&mut pack_upload, sender, receiver);

    pack_upload
}

fn start_upload_pack_process_thread(pack_upload: &mut Child, sender: Sender<String>, receiver: Receiver<String>) {
    let mut stdin = pack_upload.stdin.take().expect("Failed to get pack upload input stream");
    let stdout = pack_upload.stdout.take().expect("Failed to get pack upload output");

    thread::spawn(move || {
        let mut f = BufReader::new(stdout);
        loop {
            match receiver.try_recv() {
                Ok(line) => {
                    // XXX reads from stdin and writes to stdout
                    println!("writing: {line}");
                    stdin.write_all(line.as_bytes()).expect("Failed to write to pack upload input stream");
                }
                Err(TryRecvError::Empty) => {
                    sleep(Duration::from_secs(1));
                    continue;
                }
                Err(e) => {
                    println!("Error: {:?}", e);
                }
            }
            let mut buf = String::new();
            match f.read_line(&mut buf) {
                Ok(_) => {
                    sender.send(buf).unwrap();
                    continue;
                }
                Err(e) => {
                    println!("an error!: {:?}", e);
                    break;
                }
            }
        }
    });
}

fn start_upload_pack_command_thread(mutex: Mutex<Sender<String>>) {
    thread::spawn(move || {
        let sender = mutex.lock().unwrap();
        sender.send(String::from("dont know what to send from command thread")).unwrap();
    });
}

fn read_line(line: String) -> String {
    //let size = usize::from_str_radix(&l[0..4], 16).unwrap();
    let line = &line[4..line.len()];
    println!("READING: {line}");
    // TODO: implement ack nack processing
    return String::from(line)
}

fn write_message(want: &str, have: Option<&str>, sender_channel: &Sender<String>) {
    write_pack_line(&format!("want {}", want), sender_channel);
    write_pack_line("", sender_channel);
    match have {
        Some(have) => {
            write_pack_line(&format!("have {}", have), sender_channel);
            write_pack_line("", sender_channel);
        },
        None => {}
    }
    write_pack_line("done", sender_channel);
}

fn write_pack_line(line: &str, sender_channel: &Sender<String>) {
    if "".eq(line) {
        sender_channel.send(String::from("0000\n")).unwrap()
    } else {
        let message = format!("{0:04}{1}\n", line.len() + 4 + 1, line);
        sender_channel.send(message).unwrap();
    };
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

