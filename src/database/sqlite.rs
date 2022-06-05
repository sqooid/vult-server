use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::util::types::GenericResult;

use super::traits::{VultCacheDatabase, VultStoreDatabase};

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

impl VultStoreDatabase for SqliteDatabase {
    fn create_user_store(&self, key: &str) -> GenericResult {
        let mut path: PathBuf = self.directory.clone();
        fs::create_dir_all(&path)?;
        path.push(format!("{}.db.sqlite", key));
        let _db = sqlite::open(&path)?;
        Ok(())
    }

    fn has_user_store(&self, key: &str) -> bool {
        let mut path: PathBuf = self.directory.clone();
        path.push(format!("{}.db.sqlite", key));
        path.exists()
    }
}

impl VultCacheDatabase for SqliteDatabase {
    fn create_user_cache(&self, key: &str) -> GenericResult {
        let mut path: PathBuf = self.directory.clone();
        fs::create_dir_all(&path)?;
        path.push(format!("{}.cc.sqlite", key));
        let _db = sqlite::open(&path)?;
        Ok(())
    }

    fn has_user_cache(&self, key: &str) -> bool {
        let mut path: PathBuf = self.directory.clone();
        path.push(format!("{}.cc.sqlite", key));
        path.exists()
    }
}
