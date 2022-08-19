use anyhow::Result;

use crate::{
    api::db_types::{Credential, Mutation},
    util::types::GenericResult,
};

pub trait StoreDatabase {
    /// Apply a mutation to the store of the user of `key`
    ///
    /// Returns true if the mutation resulted in a change, false if no changes were made
    fn apply_mutation(&self, alias: &str, mutation: &Mutation) -> Result<Option<String>>;

    /// Export the entire store of the user of `key` as a list of credentials
    fn export_all(&self, alias: &str) -> GenericResult<Vec<Credential>>;

    /// Imports entire list of credentials into what should be an empty store
    fn import_all(&self, alias: &str, credentials: &[Credential]) -> GenericResult<()>;

    /// Check if database is empty for user of 'key'
    fn is_empty(&self, alias: &str) -> GenericResult<bool>;
}

pub trait CacheDatabase {
    /// Adds a list of mutations to the cache of the user of `key`
    ///
    /// Returns the `id` of the newly cached state
    /// which can be used to sync efficiently
    fn add_mutations(&self, alias: &str, mutations: &[Mutation]) -> Result<String>;

    /// Check if cache contains state id
    fn has_state(&self, alias: &str, state: &str) -> GenericResult<bool>;

    /// Get all mutations necessary to get to most up-to-date state from state `id`
    ///
    /// If `id` refers to the most current state, result is an empty list.
    fn get_next_mutations(&self, alias: &str, id: &str) -> GenericResult<Vec<Mutation>>;

    /// Check if database is empty for user of 'key'
    fn is_empty(&self, alias: &str) -> GenericResult<bool>;
}
pub trait SaltDatabase {
    fn set_salt(&self, alias: &str, salt: &str) -> Result<()>;
    fn get_salt(&self, alias: &str) -> Result<String>;
    fn remove_salt(&self, alias: &str) -> Result<()>;
}

pub struct Databases {
    pub store: Box<dyn StoreDatabase + Send + Sync>,
    pub cache: Box<dyn CacheDatabase + Send + Sync>,
    pub salt: Box<dyn SaltDatabase + Send + Sync>,
}

impl Databases {
    pub fn new(
        store: Box<dyn StoreDatabase + Send + Sync>,
        cache: Box<dyn CacheDatabase + Send + Sync>,
        salt: Box<dyn SaltDatabase + Send + Sync>,
    ) -> Self {
        Self { store, cache, salt }
    }
}
