use std::{fmt::Display, fs::File, io::Read, path::Path};

use serde::Deserialize;

use crate::util::{error::Error, types::GenericResult};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub users: Vec<User>,
    #[serde(default = "default_cache_count")]
    pub cache_count: u32,
    #[serde(default = "default_db_directory")]
    pub db_directory: String,
}

fn default_cache_count() -> u32 {
    100
}

fn default_db_directory() -> String {
    String::from("./data")
}

#[derive(Debug, Deserialize)]
pub struct User {
    pub alias: Option<String>,
    pub key: String,
}

impl Config {
    pub fn read_config<T: AsRef<Path> + Display>(path: T) -> GenericResult<Config> {
        let mut config_file = File::open(&path).map_err(|_e| Error::Config {
            message: format!("Failed to find config file {path}"),
        })?;
        let mut contents = String::new();
        config_file
            .read_to_string(&mut contents)
            .map_err(|_e| Error::Config {
                message: format!("Failed to read config file {path}"),
            })?;

        let parsed = toml::from_str(&contents)?;

        Ok(parsed)
    }
}
