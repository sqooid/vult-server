use crate::{
    api::db_types::{Credential, Mutation},
    util::types::GenericResult,
};

use super::error::DbError;

pub trait StoreDatabase {
    /// Apply a mutation to the store of the user of `key`
    fn apply_mutation(&self, key: &str, mutation: Mutation) -> Result<(), DbError>;

    /// Export the entire store of the user of `key` as a list of credentials
    fn export_all(&self, key: &str) -> Vec<Credential>;
}

pub trait CacheDatabase {
    /// Add a mutation to the cache of the user of `key`
    ///
    /// Returns the `id` of the newly cached state
    /// which can be used to sync efficiently
    fn add_mutation(&self, key: &str, mutation: Mutation) -> GenericResult<String>;

    /// Get all mutations necessary to get to most up-to-date state from state `id`
    ///
    /// If `id` refers to the most current state, result is an empty list.
    /// If `id` is not found at all, returns None
    fn get_next_mutations(&self, key: &str, id: &str) -> Option<Vec<Mutation>>;
}

pub struct Databases {
    store: Box<dyn StoreDatabase + Send + Sync>,
    cache: Box<dyn CacheDatabase + Send + Sync>,
}

impl Databases {
    pub fn new(
        store: Box<dyn StoreDatabase + Send + Sync>,
        cache: Box<dyn CacheDatabase + Send + Sync>,
    ) -> Self {
        Self { store, cache }
    }

    pub fn store<'a>(&'a self) -> &'a dyn StoreDatabase {
        self.store.as_ref()
    }

    pub fn cache<'a>(&'a self) -> &'a dyn CacheDatabase {
        self.cache.as_ref()
    }
}
