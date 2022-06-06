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

impl StoreDatabase for SqliteDatabase {
    fn create_user_store(&self, key: &str) -> GenericResult {
        let mut path: PathBuf = self.directory.clone();
        fs::create_dir_all(&path)?;
        path.push(format!("{}.store.sqlite", key));
        let _db = sqlite::open(&path)?;
        Ok(())
    }

    fn has_user_store(&self, key: &str) -> bool {
        let mut path: PathBuf = self.directory.clone();
        path.push(format!("{}.store.sqlite", key));
        path.exists()
    }
}

impl CacheDatabase for SqliteDatabase {
    fn create_user_cache(&self, key: &str) -> GenericResult {
        let mut path: PathBuf = self.directory.clone();
        fs::create_dir_all(&path)?;
        path.push(format!("{}.cache.sqlite", key));
        let _db = sqlite::open(&path)?;
        Ok(())
    }

    fn has_user_cache(&self, key: &str) -> bool {
        let mut path: PathBuf = self.directory.clone();
        path.push(format!("{}.cache.sqlite", key));
        path.exists()
    }
}
