use std::path::PathBuf;
use std::collections::HashSet;
use tokio::fs::{File, create_dir_all};
use tokio::io::{AsyncReadExt, AsyncWriteExt, ErrorKind};

// manage content of `dir/.gtr/gtrd-export
static SETTINGS_DIR: &str = ".gtr";
static SETTINGS_FILE: &str = "gtrd-export";

/// Add branches to be shared via gtrd
///
/// The first parameter is the git repo directory. The second parameter is the list of branches to be added.
/// It adds branches resolving duplication, stores them .gtr/gtrd-export.
pub async fn include(dir: &PathBuf, new_branches: &Vec<&String>) {
    let old_branches = read_old_branches(dir).await;
    let old_branches: HashSet<&String> = old_branches.iter().collect();
    let new_branches: HashSet<&String> = new_branches.iter().map(|s| *s).collect();
    let final_branches: Vec<&String> = old_branches
        .union(&new_branches)
        .into_iter()
        .map(|b| *b)
        .collect();
    write_new_branches(dir, &final_branches).await;
}

/// Removes branches to be shared via gtrd
///
/// The first parameter is the git repo directory. The second parameter is the list of branches not to be shared.
/// It removes branches resolving duplication, stores new settings in .gtr/gtrd-export.
pub async fn remove(dir: &PathBuf, del_branches: &Vec<&String>) {
    let old_branches = read_old_branches(dir).await;
    let old_branches: HashSet<&String> = old_branches.iter().collect();
    let del_branches: HashSet<&String> = del_branches.iter().map(|s| *s).collect();
    let final_branches: Vec<&String> = old_branches
        .difference(&del_branches)
        .into_iter()
        .map(|b| *b)
        .collect();
    write_new_branches(dir, &final_branches).await;
}

/// Lists branches currently shared via gtrd
///
/// The parameter is the git repo directory. It reads branches stored in .gtr/gtrd-export
pub async fn list(dir: &PathBuf) -> Vec<String>{
    read_old_branches(dir).await
}

// TODO: implement method which will guarantee that `gtd` is running on startup
// LINUX: systemd
// MACOS: launchd
// WINDOWS: task scheduler

async fn read_old_branches(dir: &PathBuf) -> Vec<String> {
    let settings_dir = dir.join(SETTINGS_DIR);
    let settings_path = settings_dir.join(SETTINGS_FILE);

    match tokio::fs::File::open(&settings_path).await {
        Ok(mut file) => {
            let mut data = String::new();
            file.read_to_string(&mut data).await.expect("Can not read file content");
            return data
                .split("\n")
                .into_iter()
                .map(|s| String::from(s)).filter(|s| !s.eq(""))
                .collect();
        },
        Err(e) => {
            match e.kind() {
                ErrorKind::NotFound => {
                    create_dir_all(&settings_dir).await.expect("Can not create gtr directory");
                    File::create(&settings_path).await.expect("Can not create settings file");
                    return vec!(String::from(""))
                },
                _ => panic!("Unrecognized error {e}")
            }
        }
    };
}

async fn write_new_branches(dir: &PathBuf, branches: &Vec<&String>) {
    let mut sorted = branches.to_vec();
    sorted.sort();
    let stred: Vec<&str> = sorted.iter().map(|b| b.as_str()).collect();

    let settings_path = dir.join(SETTINGS_DIR).join(SETTINGS_FILE);
    match File::create(&settings_path).await {
        Ok(mut file) => file.write_all(stred.join("\n").as_bytes()).await.unwrap(),
        Err(e) => panic!("Cant store settings to file {e}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn adds_entries_to_settings() {
        let mut dir = PathBuf::new();
        dir.push(".");

        let mut branches: Vec<String> = vec!["testA", "testB"]
            .iter()
            .map(|s| String::from(*s))
            .collect();
        branches.sort();
        let mut more_branches: Vec<String> = vec!["testC", "testB", "testD"]
            .iter()
            .map(|s| String::from(*s))
            .collect();
        more_branches.sort();

        let mut res_branches: Vec<String> = vec!["testA", "testB", "testC", "testD"]
            .iter()
            .map(|s| String::from(*s))
            .collect();
        res_branches.sort();

        let input_branches: Vec<&String> = branches.iter().collect();
        include(&dir, &input_branches).await;
        let res = read_old_branches(&dir).await;
        println!("{res:#?}, {branches:#?}");
        assert!(res.eq(&branches));

        let input_branches: Vec<&String> = more_branches.iter().collect();
        include(&dir, &input_branches).await;
        assert!(read_old_branches(&dir).await.eq(&res_branches));

        let input_branches: Vec<&String> = res_branches.iter().collect();
        remove(&dir, &input_branches).await;
        assert!(read_old_branches(&dir).await.join("").eq(""));
    }
}
