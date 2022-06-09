use std::time::SystemTime;
use std::{fs, path::PathBuf};

use crate::api::db_types::Mutation;
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
        db.execute("create table if not exists Store (id text primary key, value text)")?;
        Ok(db)
    }

    fn open_cache(&self, key: &str) -> GenericResult<sqlite::Connection> {
        let db = self.open_db(key)?;
        db.execute("create table if not exists Cache (id text primary key, mutation blob)")?;
        Ok(db)
    }
}

impl StoreDatabase for SqliteDatabase {
    fn apply_mutation(&self, key: &str, mutation: &Mutation) -> Result<(), Error> {
        todo!()
    }

    fn export_all(&self, key: &str) -> Vec<crate::api::db_types::Credential> {
        todo!()
    }
}

impl CacheDatabase for SqliteDatabase {
    fn add_mutation(&self, key: &str, mutation: &Mutation) -> GenericResult<String> {
        let time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_millis();
        let id = time.to_string();

        let mutation_blob = bincode::serialize(&mutation)?;

        let db = self.open_cache(&key)?;
        let mut statement = db.prepare(format!("insert into Cache values ({}, ?)", &id))?;
        statement.bind(1, mutation_blob.as_slice())?;
        statement.next()?;

        Ok(id)
    }

    fn get_next_mutations(&self, key: &str, id: &str) -> GenericResult<Vec<Mutation>> {
        let db = self.open_cache(&key)?;
        let mut cursor = db
            .prepare("select * from Cache where id > ?")?
            .into_cursor();
        cursor.bind(&[sqlite::Value::String(id.into())])?;

        let mut mutations: Vec<Mutation> = Vec::new();
        while let Some(row) = cursor.next()? {
            let bin = row[1].as_binary().ok_or(Error::Unknown {
                message: "Cache database values scuffed".into(),
            })?;
            let mutation: Mutation = bincode::deserialize(&bin)?;
            mutations.push(mutation);
        }

        Ok(mutations)
    }
}
