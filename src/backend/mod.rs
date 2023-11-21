use crate::{Innerable, Result};

#[cfg(feature = "unstable_any")]
/// Any database backend (requires `unstable_any` feature)
pub mod any;

#[cfg(feature = "memdb")]
/// In-memory database backend (requires `memdb` feature)
pub mod mem;

#[cfg(feature = "rocksdb")]
/// RocksDB database backend (requires `rocksdb` feature)
pub mod rocksdb;

pub trait DatabaseBackend<'a>: Innerable + Sized + Send + Sync + 'a {
    /// The type of the 'column', this is a reference to a database.
    type Column: DatabaseColumn;
    fn create_or_open(&'a self, db: &str) -> Result<Self::Column>;
}

pub trait DatabaseColumn {
    fn set(&self, key: impl AsRef<[u8]>, val: &[u8]) -> Result<()>;
    fn get(&self, key: impl AsRef<[u8]>) -> Result<Option<Vec<u8>>>;
    fn get_multi<I>(&self, keys: I) -> Result<Vec<Option<Vec<u8>>>>
    where
        I: IntoIterator,
        I::Item: AsRef<[u8]>;
    fn delete(&self, key: impl AsRef<[u8]>) -> Result<()>;
    fn contains(&self, key: impl AsRef<[u8]>) -> Result<bool>;
    fn clear(&self) -> Result<()>;
}

pub trait DatabaseColumnRef<'c>: DatabaseColumn {
    type Ref: AsRef<[u8]> + 'c + std::ops::Deref<Target = [u8]> + Send + Sync;
    fn get_ref(&self, key: impl AsRef<[u8]>) -> Result<Option<Self::Ref>>;
    fn get_multi_ref<I>(&self, keys: I) -> Result<Vec<Option<Self::Ref>>>
    where
        I: IntoIterator,
        I::Item: AsRef<[u8]>;
}

pub trait DatabaseTxn: DatabaseColumn {
    fn commit(&self) -> Result<()>;
    fn rollback(&self) -> Result<()>;
}

pub trait DatabaseColumnTxn<'c>: DatabaseColumn {
    type Txn: DatabaseTxn;
    fn transaction(&self) -> Result<Self::Txn>;
}
