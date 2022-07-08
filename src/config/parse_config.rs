use std::{fmt::Display, fs::File, io::Read, path::Path};

use serde::{Deserialize, Serialize};

use crate::util::{error::Error, types::GenericResult};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub users: Vec<User>,
    #[serde(default = "default_cache_count")]
    pub cache_count: u32,
    #[serde(default = "default_db_directory")]
    pub db_directory: String,
}

impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string_pretty(self).map_err(|_| std::fmt::Error)?
        )
    }
}

fn default_cache_count() -> u32 {
    100
}

fn default_db_directory() -> String {
    String::from("./data")
}

#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    pub alias: String,
    pub keys: Vec<String>,
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
