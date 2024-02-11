use bincode;
pub use common::rule::Rule;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::{io, sync::Mutex};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub root: String,
    pub password: String,
    pub port: u16,
    pub listen_addr: String,
    pub rules: Vec<Rule>,
}

const CONFIG_FILE_PATH: &str = "config.json";
lazy_static! {
    pub(crate) static ref CONFIG: Mutex<Config> = Mutex::new(Config::load_from_file());
}
impl Config {
    pub fn new(
        root: String,
        password: String,
        port: u16,
        host: String,
        rules: Vec<Rule>,
    ) -> Config {
        Config {
            root,
            password,
            port,
            listen_addr: host,
            rules,
        }
    }

    pub fn init() -> Config {
        let root = "root".to_string();
        let password = "password".to_string();
        let port = 8080;
        let host = "127.0.0.1".to_string();
        let rules = vec![];
        Config::new(root, password, port, host, rules)
    }

    pub fn load_from_file() -> Config {
        let data = std::fs::read_to_string(CONFIG_FILE_PATH).unwrap_or_else(|_| {
            let config = Config::init();
            config.save_to_file().unwrap();
            config.to_json().unwrap()
        });
        Config::from_json(data).unwrap()
    }

    pub fn save_to_file(&self) -> io::Result<()> {
        let data = serde_json::to_string_pretty(&self).unwrap();
        std::fs::write(CONFIG_FILE_PATH, data)
    }

    pub fn to_u8(&self) -> bincode::Result<Vec<u8>> {
        bincode::serialize(&self)
    }
    pub fn from_u8(data: Vec<u8>) -> bincode::Result<Config> {
        bincode::deserialize(&data)
    }
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(&self)
    }
    pub fn from_json(data: String) -> serde_json::Result<Config> {
        serde_json::from_str(&data)
    }
}
