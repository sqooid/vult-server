use std::time::SystemTime;
use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use rusqlite::params;

use crate::api::db_types::{Credential, DbMutation, Mutation};
use crate::util::error::Error;
use crate::util::id::random_b64;
use crate::util::types::GenericResult;

use super::traits::{CacheDatabase, StoreDatabase, UserDatabase};

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
            "create table if not exists Cache (id text primary key, time integer, mutation blob)",
            [],
        )?;
        Ok(db)
    }

    fn open_user(&self) -> GenericResult<rusqlite::Connection> {
        let db = self.open_db("vult.internal")?;
        db.execute(
            "create table if not exists User (alias text primary key, salt text, hash text)",
            [],
        )?;
        Ok(db)
    }
}

impl StoreDatabase for SqliteDatabase {
    fn apply_mutation(&self, alias: &str, mutation: &Mutation) -> Result<Option<String>> {
        let db = self.open_store(alias)?;

        match mutation {
            Mutation::Add { credential } => {
                info!("Applying add {}", &credential.id);
                let result = db.execute(
                    "insert into Store values (?, ?)",
                    [&credential.id, &credential.value],
                );
                match result {
                    Ok(_) => Ok(None),
                    Err(rusqlite::Error::SqliteFailure(e, _)) => {
                        if e.extended_code == 1555 {
                            let mut new_id;
                            while {
                                new_id = random_b64(24);
                                match db.execute(
                                    "insert into Store values (?, ?)",
                                    [&new_id, &credential.value],
                                ) {
                                    Ok(_) => false,
                                    Err(rusqlite::Error::SqliteFailure(e, _)) => {
                                        e.extended_code == 1555
                                    }
                                    Err(_) => {
                                        result.context("Failed to assign new id to credental with duplicated id")?;
                                        unreachable!()
                                    }
                                }
                            } {}
                            Ok(Some(new_id))
                        } else {
                            result.with_context(|| {
                                format!("Failed to add credential to store {}", &credential)
                            })?;
                            unreachable!()
                        }
                    }
                    Err(_) => {
                        result.with_context(|| {
                            format!("Failed to add credential to store {}", &credential)
                        })?;
                        unreachable!()
                    }
                }
            }

            Mutation::Delete { credential } => {
                info!("Applying delete {}", &credential.id);
                let result = db.execute("delete from Store where id = ?", [&credential.id]);
                match result {
                    Ok(1) => Ok(None),
                    Ok(_) => Err(Error::MissingId(credential.id.to_owned()).into()),
                    Err(_) => {
                        result.with_context(|| {
                            format!("Failed to delete credential with id {}", credential.id)
                        })?;
                        unreachable!()
                    }
                }
            }
            Mutation::Modify { credential } => {
                info!("Applying modify {}", &credential.id);
                let result = db.execute(
                    "update Store set value = ? where id = ?",
                    [&credential.value, &credential.id],
                );
                match result {
                    Ok(1) => Ok(None),
                    Ok(_) => Err(Error::MissingId(credential.id.to_owned()).into()),
                    Err(_) => {
                        result.with_context(|| {
                            format!("Failed to modify credential {}", credential)
                        })?;
                        unreachable!()
                    }
                }
            }
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
            return Err(Error::ExistingUser(alias.to_string()));
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
    fn add_mutations(&self, alias: &str, mutations: &[Mutation]) -> Result<String> {
        let time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_nanos();

        let mutations: &[DbMutation] = unsafe {
            std::slice::from_raw_parts(mutations.as_ptr() as *const DbMutation, mutations.len())
        };
        let mutation_blob = bincode::serialize(&mutations)?;

        let db = self.open_cache(alias)?;

        let mut id;
        while {
            id = random_b64(24);
            let result = db.execute(
                "insert into Cache values (?, ?, ?)",
                params![id, time as u64, mutation_blob],
            );
            match &result {
                Ok(_) => false,
                Err(rusqlite::Error::SqliteFailure(e, _)) => {
                    if e.extended_code == 1555 {
                        true
                    } else {
                        result.context("Failed to add mutations to database")?;
                        unreachable!();
                    }
                }
                _ => {
                    result.context("Failed to add mutations to database")?;
                    unreachable!();
                }
            }
        } {}

        Ok(id)
    }

    fn get_next_mutations(&self, alias: &str, id: &str) -> GenericResult<Vec<Mutation>> {
        let mut mutations: Vec<Mutation> = Vec::new();

        let db = self.open_cache(alias)?;
        let mut statement = db.prepare(
            "select mutation from Cache where time > (select time from Cache where id = ?) order by time desc",
        )?;
        let mutation_blob_iter = statement.query_map([id], |row| {
            let mutation: Vec<u8> = row.get(0)?;
            Ok(mutation)
        })?;

        for mutation_blob in mutation_blob_iter.flatten() {
            let mutation: Vec<DbMutation> = bincode::deserialize(&mutation_blob)?;
            let mut ptr = std::mem::ManuallyDrop::new(mutation);
            let mut mutation: Vec<Mutation> = unsafe {
                Vec::from_raw_parts(ptr.as_mut_ptr() as *mut Mutation, ptr.len(), ptr.len())
            };
            mutations.append(&mut mutation);
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

    fn has_state(&self, alias: &str, state: &str) -> GenericResult<bool> {
        let db = self.open_cache(alias)?;
        let mut statement = db.prepare("select id from Cache where id = ?")?;
        match statement.query_row([state], |_| Ok(())) {
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(false),
            Ok(_) => Ok(true),
            Err(e) => Err(e.into()),
        }
    }
}

impl UserDatabase for SqliteDatabase {
    fn add_user(&self, alias: &str, salt: &str, hash: &str) -> Result<()> {
        let db = self.open_user()?;
        db.execute("insert into User values (?, ?, ?)", [alias, salt, hash])
            .context("Failed to insert salt into database")?;
        Ok(())
    }

    fn get_user(&self, alias: &str) -> Result<(String, String)> {
        let db = self.open_user()?;
        let mut statement = db.prepare("select salt, hash from User where alias = ?")?;
        match statement.query_row([alias], |row| {
            let salt: String = row.get(0)?;
            let hash: String = row.get(1)?;
            Ok((salt, hash))
        }) {
            Ok(value) => Ok(value),
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                Err(Error::UninitializedUser(alias.to_string()).into())
            }
            Err(e) => Err(Error::Server(e.into()).into()),
        }
    }

    fn remove_salt(&self, alias: &str) -> Result<()> {
        let db = self.open_user()?;
        db.execute("delete from User where alias = ?", [alias])
            .context("Failed to delete salt")
            .map(|_| ())
    }
}
