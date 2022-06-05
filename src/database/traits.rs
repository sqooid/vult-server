use crate::util::types::GenericResult;

pub trait VultStoreDatabase {
    fn create_user_store(&self, key: &str) -> GenericResult;
    fn has_user_store(&self, key: &str) -> bool;
}

pub trait VultCacheDatabase {
    fn create_user_cache(&self, key: &str) -> GenericResult;
    fn has_user_cache(&self, key: &str) -> bool;
}
