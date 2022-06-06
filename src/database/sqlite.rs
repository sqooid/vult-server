use std::{fs, path::PathBuf};

use crate::util::types::GenericResult;

use super::traits::{CacheDatabase, StoreDatabase};

pub struct SqliteDatabase {
    directory: PathBuf,
}

impl SqliteDatabase {
    pub fn new<D: Into<PathBuf>>(directory: D) -> Self {
        Self {
            directory: directory.into(),
        }
    }
}

fn get_db_path(key: &str) -> String {
    format!("{}.sqlite", key)
}

impl StoreDatabase for SqliteDatabase {
    fn create_user_store(&self, key: &str) -> GenericResult {
        let mut path: PathBuf = self.directory.clone();
        fs::create_dir_all(&path)?;
        path.push(get_db_path(key));
        let _db = sqlite::open(&path)?;
        Ok(())
    }

    fn has_user_store(&self, key: &str) -> bool {
        let mut path: PathBuf = self.directory.clone();
        path.push(get_db_path(key));
        path.exists()
    }
}

impl CacheDatabase for SqliteDatabase {
    fn create_user_cache(&self, key: &str) -> GenericResult {
        let mut path: PathBuf = self.directory.clone();
        fs::create_dir_all(&path)?;
        path.push(get_db_path(key));
        let _db = sqlite::open(&path)?;
        Ok(())
    }

    fn has_user_cache(&self, key: &str) -> bool {
        let mut path: PathBuf = self.directory.clone();
        path.push(get_db_path(key));
        path.exists()
    }
}
