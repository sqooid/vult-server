use std::{fs, path::PathBuf};

use crate::util::types::GenericResult;

use super::traits::{CacheDatabase, StoreDatabase};

pub struct SqliteDatabase {
    directory: PathBuf,
}

fn get_db_path(key: &str) -> String {
    format!("{}.sqlite", key)
}

impl SqliteDatabase {
    pub fn new<D: Into<PathBuf>>(directory: D) -> Self {
        Self {
            directory: directory.into(),
        }
    }

    fn open_db(&self, key: &str) -> GenericResult<sqlite::Connection> {
        let mut path: PathBuf = self.directory.clone();
        if !path.exists() {
            fs::create_dir_all(&path)?;
        }
        path.push(get_db_path(key));
        let db = sqlite::open(&path)?;
        Ok(db)
    }

    fn open_store(&self, key: &str) -> GenericResult<sqlite::Connection> {
        let db = self.open_db(key)?;
        db.execute("create table if not exists Store ()")?;
        Ok(db)
    }

    fn open_cache(&self, key: &str) -> GenericResult<sqlite::Connection> {
        let db = self.open_db(key)?;
        db.execute("create table if not exists Cache ()")?;
        Ok(db)
    }
}

impl StoreDatabase for SqliteDatabase {}

impl CacheDatabase for SqliteDatabase {}
