use std::path::PathBuf;
use tokio::fs::{File, create_dir_all};
use tokio::io::{AsyncReadExt, AsyncWriteExt, ErrorKind};
use toml;

// manage content of `dir/.gtr/gtrd-export
static CONFIG_DIR: &str = ".gtr";
static CONFIG_FILE: &str = "config.toml";

use serde::{Serialize, Deserialize};
#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub branches: Vec<String>,
}

impl Config {
    pub async fn store(self, dir: &PathBuf) {
        // TODO: move out?
        let config_dir = dir.join(CONFIG_DIR);
        let settings_path = config_dir.join(CONFIG_FILE);

        match File::create(&settings_path).await {
            Err(e) => panic!("Cant store settings to file {e}"),
            Ok(mut file) => {
                let content = toml::to_string(&self).unwrap();
                file.write_all(content.as_bytes()).await.unwrap();
            }
        }
    }
}
pub async fn read_or_create(dir: &PathBuf) -> Config {
    // TODO: move out?
    let config_dir = dir.join(CONFIG_DIR);
    let settings_path = config_dir.join(CONFIG_FILE);
    let default_config = Config {
        branches: vec![]
    };

    match tokio::fs::File::open(&settings_path).await {
        Ok(mut file) => {
            let mut data = String::new();
            file.read_to_string(&mut data).await.expect("Can not read file content");

            return toml::from_str(&data).unwrap_or(default_config)
        },
        Err(e) => {
            match e.kind() {
                ErrorKind::NotFound => {
                    create_dir_all(&config_dir).await.expect("Can not create gtr directory");
                    default_config.store(dir).await;

                    let default_config = Config {
                        branches: vec![]
                    };
                    return default_config
                },
                _ => panic!("Unrecognized error {e}")
            }
        }
    };
}
