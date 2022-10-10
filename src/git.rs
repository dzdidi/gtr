use std::io::{Read, ErrorKind, Write};
use std::process::Command;
use std::str;
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};

static SETTINGS_DIR: &str = ".gtr";

pub struct Ref {
    pub hash: String,
    pub referrence: String,
}

impl Ref {
    fn new(hash: &str, referrence: &str) -> Self {
        Self {
            hash: String::from(hash),
            referrence: String::from(referrence)
        }
    }
}

pub fn is_git(dir: &str) {
    if !Path::new(dir).join(".git").exists() {
        panic!("Not a git repository");
    }
}

// also returns HEAD
pub fn ls_remote(dir: &str) -> Vec<Ref> {
    let refs = Command::new("git").arg("ls-remote").arg(dir).output().unwrap();
    let mut result = Vec::new();
    let refs = str::from_utf8(&refs.stdout[..]).unwrap();
    refs.split("\n").into_iter().for_each(|r| {
        if r == "\n" || r == "" { return }

        let s: Vec<&str> = r.split("\t").collect();
        result.push(Ref::new(s[0], s[1]))
    });

    return result
}

pub fn ignore_gtr(dir: &str) {
    let gitignore_path = Path::new(dir).join(".gitignore");
    match File::open(&gitignore_path) {
        Ok(mut file) => {
            let mut data = String::new();
            file.read_to_string(&mut data).expect("Can not read file content");
            let gtr_ignored = data.replace("\n", " ")
                .split(" ")
                .into_iter()
                .any(|s| String::from(SETTINGS_DIR).eq(s));

            if !gtr_ignored {
                create_gtr_ignore(&gitignore_path);
            }
        },
        Err(e) => {
            match e.kind() {
                ErrorKind::NotFound => {
                    File::create(&gitignore_path).unwrap();
                    create_gtr_ignore(&gitignore_path)
                },
                _ => panic!("Unrecognized error {e}")
            }
        }
    }
}

fn create_gtr_ignore(gitignore_path: &PathBuf) {
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(gitignore_path)
        .unwrap();
    file.write_all((String::from("\n") + SETTINGS_DIR).as_bytes()).unwrap();
}
