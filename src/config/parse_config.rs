use std::{fs::File, io::Read, path::Path};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub users: Vec<User>,
    #[serde(default = "default_cache_count")]
    pub cache_count: u32,
}

fn default_cache_count() -> u32 {
    100
}

#[derive(Debug, Deserialize)]
pub struct User {
    pub alias: Option<String>,
    pub key: String,
}

pub fn read_config<T: AsRef<Path>>(path: T) -> Result<Config, Box<dyn std::error::Error>> {
    let mut config_file = File::open(path)?;
    let mut contents = String::new();
    config_file.read_to_string(&mut contents)?;

    let parsed = toml::from_str(&contents)?;

    Ok(parsed)
}
