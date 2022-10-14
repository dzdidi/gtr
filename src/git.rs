use std::collections::HashMap;
use std::io::{Read, ErrorKind, Write};
use std::process::Command;
use std::str;
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};

static SETTINGS_DIR: &str = ".gtr";

/// Panics if provided directory is not a git repository
pub fn is_git(dir: &str) {
    if !Path::new(dir).join(".git").exists() {
        panic!("Not a git repository");
    }
}

/// Returns hash of Ref for each branch of given repository as well as current HEAD
pub fn ls_remote(dir: &str) -> HashMap<String, String> {
    let refs = Command::new("git").arg("ls-remote").arg(dir).output().unwrap();
    let refs = str::from_utf8(&refs.stdout[..]).unwrap();
    return refs
        .split("\n")
        .into_iter()
        .filter(|r| !String::from("\n").eq(r) && !String::from("").eq(r))
        .map(|r| {
            let s: Vec<&str> = r.split("\t").collect();
            return (String::from(s[0]), String::from(s[1]))
        })
        .collect();

}

/// Add .gtr directory to gitignore in provided repository
pub fn ignore_gtr(dir: &str) {
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
