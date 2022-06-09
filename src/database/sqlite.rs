use std::time::SystemTime;
use std::{fs, path::PathBuf};

use rusqlite::params;
use rusqlite::types::ToSqlOutput;

use crate::api::db_types::{Credential, Mutation};
use crate::util::error::Error;
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

    fn open_db(&self, key: &str) -> GenericResult<rusqlite::Connection> {
        let mut path: PathBuf = self.directory.clone();
        if !path.exists() {
            fs::create_dir_all(&path)?;
        }
        path.push(get_db_path(key));
        let db = rusqlite::Connection::open(&path)?;
        Ok(db)
    }

    fn open_store(&self, key: &str) -> GenericResult<rusqlite::Connection> {
        let db = self.open_db(key)?;
        db.execute("create table if not exists Store (id text primary key, value text)", [])?;
        Ok(db)
    }

    fn open_cache(&self, key: &str) -> GenericResult<rusqlite::Connection> {
        let db = self.open_db(key)?;
        db.execute("create table if not exists Cache (id text primary key, mutation blob)", [])?;
        Ok(db)
    }
}

impl StoreDatabase for SqliteDatabase {
    fn apply_mutation(&self, key: &str, mutation: &Mutation) -> GenericResult<()> {
        let db = self.open_cache(&key)?;

        match mutation {
            Mutation::Add { credential } => {
                let mut statement = db.prepare("insert into Store values")
            }
        }
    }

    fn export_all(&self, key: &str) -> Vec<Credential> {
        todo!()
    }

    fn import_all(&self, key: &str, credentials: &[Credential]) -> GenericResult<()> {
        todo!()
    }
}

impl CacheDatabase for SqliteDatabase {
    fn add_mutation(&self, key: &str, mutation: &Mutation) -> GenericResult<String> {
        let time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_millis();
        let id = time.to_string();

        let mutation_blob = bincode::serialize(mutation)?;

        let db = self.open_cache(&key)?;
        let mut statement = db.execute("insert into Cache values (?, ?)", params![id, mutation_blob])?;

        Ok(id)
    }

    fn get_next_mutations(&self, key: &str, id: &str) -> GenericResult<Vec<Mutation>> {
        let mut mutations: Vec<Mutation> = Vec::new();

        let db = self.open_cache(&key)?;
        let mut statement = db
            .prepare("select * from Cache where id > ?")?;
        let mutation_blob_iter = statement.query_map([], |row|{
            let mutation: Vec<u8> = row.get(1)?;
            Ok(mutation)
        })?;

        for mutation_blob in mutation_blob_iter {
            if let Ok(mutation_blob) = mutation_blob {
                let mutation: Mutation = bincode::deserialize(&mutation_blob)?;
                mutations.push(mutation);
            }
        }

        Ok(mutations)
    }
}
