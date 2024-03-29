use std::{fmt::Display, fs::File, io::Read, path::Path};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::util::error::Error;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub users: Vec<User>,
    #[serde(default = "default_cache_count")]
    pub cache_count: u32,
    #[serde(default = "default_db_directory")]
    pub db_directory: String,
    #[serde(skip)]
    pub enable_test_routes: bool,
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
    pub fn read_config<T: AsRef<Path> + Display>(path: T) -> Result<Config> {
        let mut config_file = File::open(&path)
            .map_err(|e| Error::Config(e.into()))
            .with_context(|| format!("Failed to open config at path {}", &path))?;
        let mut contents = String::new();
        config_file
            .read_to_string(&mut contents)
            .map_err(|_e| Error::Config(_e.into()))
            .context("Failed to read config file")?;

        let parsed = toml::from_str(&contents).context("Failed to parse config file")?;

        Ok(parsed)
    }
}
