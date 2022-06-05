use crate::util::types::GenericResult;

pub trait StoreDatabase {
    fn create_user_store(&self, key: &str) -> GenericResult;
    fn has_user_store(&self, key: &str) -> bool;
}

pub trait CacheDatabase {
    fn create_user_cache(&self, key: &str) -> GenericResult;
    fn has_user_cache(&self, key: &str) -> bool;
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
