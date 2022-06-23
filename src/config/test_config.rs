use std::path::Path;

use super::parse_config::{Config, User};

pub fn init_test_config(dir: &str) -> Config {
    if Path::new(dir).exists() {
        std::fs::remove_dir_all(dir).expect("Remove test data directory");
    }
    std::fs::create_dir_all(dir).expect("Create test data directory");
    Config {
        users: vec![User {
            alias: None,
            key: "unit".into(),
        }],
        cache_count: 50,
        db_directory: dir.into(),
    }
}
