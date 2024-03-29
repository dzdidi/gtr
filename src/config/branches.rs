use std::path::PathBuf;
use std::collections::HashSet;
use crate::config::config_file;
use crate::utils::error::GtrResult;

/// Add branches to be shared via gtrd
///
/// The first parameter is the git repo directory. The second parameter is the list of branches to be added.
/// It adds branches resolving duplication, stores them .gtr/gtrd-export.
pub async fn include(dir: &PathBuf, new_branches: &Vec<&String>) -> GtrResult<()> {
    let old_branches = read_branches(dir).await?;
    let old_branches: HashSet<&String> = old_branches.iter().collect();
    let new_branches: HashSet<&String> = new_branches.iter().map(|s| *s).collect();
    let final_branches: Vec<&String> = old_branches
        .union(&new_branches)
        .into_iter()
        .map(|b| *b)
        .collect();
    write_new_branches(dir, &final_branches).await?;

    return Ok(())
}

/// Removes branches to be shared via gtrd
///
/// The first parameter is the git repo directory. The second parameter is the list of branches not to be shared.
/// It removes branches resolving duplication, stores new settings in .gtr/gtrd-export.
pub async fn remove(dir: &PathBuf, del_branches: &Vec<&String>) -> GtrResult<()> {
    let old_branches = read_branches(dir).await?;
    let old_branches: HashSet<&String> = old_branches.iter().collect();
    let del_branches: HashSet<&String> = del_branches.iter().map(|s| *s).collect();
    let final_branches: Vec<&String> = old_branches
        .difference(&del_branches)
        .into_iter()
        .map(|b| *b)
        .collect();
    write_new_branches(dir, &final_branches).await?;

    return Ok(())
}

/// Lists branches currently shared via gtrd
///
/// The parameter is the git repo directory. It reads branches stored in .gtr/gtrd-export
pub async fn list(dir: &PathBuf) -> GtrResult<Vec<String>>{
    let res = read_branches(dir).await?;
    Ok(res)
}

// TODO: implement method which will guarantee that `gtd` is running on startup
// LINUX: systemd
// MACOS: launchd
// WINDOWS: task scheduler

async fn read_branches(dir: &PathBuf) -> GtrResult<Vec<String>> {
    let conf = config_file::read_or_create(dir).await?;
    return Ok(conf.branches);
}

async fn write_new_branches(dir: &PathBuf, branches: &Vec<&String>) -> GtrResult<()>{
    let mut sorted = branches.to_vec();
    sorted.sort();

    let mut conf = config_file::read_or_create(dir).await?;
    conf.branches = sorted.iter().map(|b| String::from(*b)).collect();

    conf.save(dir).await?;

    return Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn adds_and_removes_branches_in_settings() {
        let mut dir = PathBuf::new();
        dir.push("./.test");

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
        include(&dir, &input_branches).await.unwrap();

        let res = read_branches(&dir).await.unwrap();
        println!("{res:#?}, {branches:#?}");
        assert!(res.eq(&branches));

        let input_branches: Vec<&String> = more_branches.iter().collect();
        include(&dir, &input_branches).await.unwrap();
        assert!(read_branches(&dir).await.unwrap().eq(&res_branches));

        let input_branches: Vec<&String> = res_branches.iter().collect();
        remove(&dir, &input_branches).await.unwrap();
        assert!(read_branches(&dir).await.unwrap().join("").eq(""));
    }
}
