use std::time::SystemTime;
use std::{fs, path::PathBuf};

use rusqlite::params;

use crate::api::db_types::{Credential, Mutation};
use crate::util::error::Error;
use crate::util::id::new_cred_id;
use crate::util::types::GenericResult;

use super::traits::{CacheDatabase, StoreDatabase};

pub struct SqliteDatabase {
    directory: PathBuf,
}

fn get_db_path(alias: &str) -> String {
    format!("{}.sqlite", alias)
}

impl SqliteDatabase {
    pub fn new<D: Into<PathBuf>>(directory: D) -> Self {
        Self {
            directory: directory.into(),
        }
    }

    fn open_db(&self, alias: &str) -> GenericResult<rusqlite::Connection> {
        let mut path: PathBuf = self.directory.clone();
        if !path.exists() {
            fs::create_dir_all(&path)?;
        }
        path.push(get_db_path(alias));
        let db = rusqlite::Connection::open(&path)?;
        Ok(db)
    }

    fn open_store(&self, alias: &str) -> GenericResult<rusqlite::Connection> {
        let db = self.open_db(alias)?;
        db.execute(
            "create table if not exists Store (id text primary key, value text)",
            [],
        )?;
        Ok(db)
    }

    fn open_cache(&self, alias: &str) -> GenericResult<rusqlite::Connection> {
        let db = self.open_db(alias)?;
        db.execute(
            "create table if not exists Cache (id text primary key, mutation blob)",
            [],
        )?;
        Ok(db)
    }
}

impl StoreDatabase for SqliteDatabase {
    fn apply_mutation(&self, alias: &str, mutation: &Mutation) -> GenericResult<()> {
        let db = self.open_store(alias)?;

        match mutation {
            Mutation::Add { credential } => match db.execute(
                "insert into Store values (?, ?)",
                [&credential.id, &credential.value],
            )? {
                1 => Ok(()),
                _ => {
                    let mut new_id = String::new();
                    while {
                        new_id = new_cred_id();
                        db.execute(
                            "insert into Store values (?, ?)",
                            [&new_id, &credential.value],
                        )? == 0
                    } {}
                    Err(Error::DuplicateId {
                        id: credential.id.to_owned(),
                        new_id: new_id,
                    })
                }
            },

            Mutation::Delete { id } => match db.execute("delete from Store where id = ?", [id])? {
                1 => Ok(()),
                _ => Err(Error::MissingItem { id: id.to_owned() }),
            },
            Mutation::Modify { credential } => match db.execute(
                "update Store set value = ? where id = ?",
                [&credential.value, &credential.id],
            )? {
                1 => Ok(()),
                _ => Err(Error::MissingItem {
                    id: credential.id.to_owned(),
                }),
            },
        }
    }

    fn export_all(&self, alias: &str) -> GenericResult<Vec<Credential>> {
        let db = self.open_store(alias)?;
        let mut statement = db.prepare("select * from Store")?;

        let iter = statement.query_map([], |row| {
            Ok(Credential {
                id: row.get(0)?,
                value: row.get(1)?,
            })
        })?;

        let mut credentials: Vec<Credential> = Vec::new();
        for credential in iter {
            credentials.push(credential?);
        }

        Ok(credentials)
    }

    fn import_all(&self, alias: &str, credentials: &[Credential]) -> GenericResult<()> {
        let db = self.open_store(alias)?;

        let mut statement = db.prepare("select * from Store limit 1")?;
        if statement.exists([])? {
            return Err(Error::ExistingUser {
                message: format!("User with alias: {alias} already exists"),
            });
        }

        let mut statement = db.prepare("insert into Store values (?, ?)")?;
        for credential in credentials {
            statement.execute([&credential.id, &credential.value])?;
        }

        Ok(())
    }

    fn is_empty(&self, alias: &str) -> GenericResult<bool> {
        let db = self.open_store(alias)?;
        let mut statement = db.prepare("select id from Store limit 1")?;
        let mut iter = statement.query_map([], |row| {
            let result: bool = row.get(1)?;
            Ok(result)
        })?;
        Ok(iter.next().is_none())
    }
}

impl CacheDatabase for SqliteDatabase {
    fn add_mutation(&self, alias: &str, mutation: &Mutation) -> GenericResult<String> {
        let time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_millis();
        let id = time.to_string();

        let mutation_blob = bincode::serialize(mutation)?;

        let db = self.open_cache(alias)?;
        db.execute(
            "insert into Cache values (?, ?)",
            params![id, mutation_blob],
        )?;

        Ok(id)
    }

    fn get_next_mutations(&self, alias: &str, id: &str) -> GenericResult<Vec<Mutation>> {
        let mut mutations: Vec<Mutation> = Vec::new();

        let db = self.open_cache(alias)?;
        let mut statement = db.prepare("select * from Cache where id > ?")?;
        let mutation_blob_iter = statement.query_map([id], |row| {
            let mutation: Vec<u8> = row.get(1)?;
            Ok(mutation)
        })?;

        for mutation_blob in mutation_blob_iter.flatten() {
            let mutation: Mutation = bincode::deserialize(&mutation_blob)?;
            mutations.push(mutation);
        }

        Ok(mutations)
    }

    fn is_empty(&self, alias: &str) -> GenericResult<bool> {
        let db = self.open_cache(alias)?;
        let mut statement = db.prepare("select id from Cache limit 1")?;
        let mut iter = statement.query_map([], |row| {
            let result: bool = row.get(1)?;
            Ok(result)
        })?;
        Ok(iter.next().is_none())
    }
}
