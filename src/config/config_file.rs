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
    pub transport: Transport,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Transport {
    pub torrent: Option<Torrent>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Torrent {
    pub router: AddressPort,
    pub bind: AddressPort,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AddressPort {
    pub addr: String,
    pub port: u16,
}

const DEFAULT_CONFIG: Config = Config {
    branches: vec![],
    transport: Transport { torrent: None }
};

impl Config {
    pub async fn save(&self, dir: &PathBuf) {
        let (_, settings_path) = get_config_path_dir_and_file(dir);

        match File::create(&settings_path).await {
            Err(e) => panic!("Cant save config to file {e}"),
            Ok(mut file) => {
                let content = toml::to_string(&self).unwrap();
                file.write_all(content.as_bytes()).await.unwrap();
            }
        }
    }
}
pub async fn read_or_create(dir: &PathBuf) -> Config {
    let (config_dir, settings_path) = get_config_path_dir_and_file(dir);
    match tokio::fs::File::open(&settings_path).await {
        Ok(mut file) => {
            let mut data = String::new();
            file.read_to_string(&mut data).await.expect("Can not read file content");

            return toml::from_str(&data).unwrap_or(DEFAULT_CONFIG)
        },
        Err(e) => {
            match e.kind() {
                ErrorKind::NotFound => {
                    create_dir_all(&config_dir).await.expect("Can not create gtr directory");
                    DEFAULT_CONFIG.save(dir).await;

                    return DEFAULT_CONFIG
                },
                _ => { panic!("Unrecognized error {e}") }
            }
        }
    };
}

fn get_config_path_dir_and_file(dir: &PathBuf) -> (PathBuf, PathBuf) {
    let config_dir = dir.join(CONFIG_DIR);
    let settings_path = config_dir.join(CONFIG_FILE);

    return (config_dir, settings_path)
}
