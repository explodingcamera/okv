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

/// Database backend trait.
pub trait DatabaseBackend<'a>: Innerable + Sized + Send + Sync + 'a {
    /// The type of the 'column', this is a reference to a database.
    type Column: DatabaseCommon;

    /// Create or open a database.
    fn create_or_open(&'a self, db: &str) -> Result<Self::Column>;
}

/// Database column trait.
pub trait DatabaseCommon {
    /// Set a key-value pair.
    fn set(&self, key: impl AsRef<[u8]>, val: &[u8]) -> Result<()>;

    /// Get a value by key.
    fn get(&self, key: impl AsRef<[u8]>) -> Result<Option<Vec<u8>>>;

    /// Get a value by key in batch.
    fn get_multi<I>(&self, keys: I) -> Result<Vec<Option<Vec<u8>>>>
    where
        I: IntoIterator,
        I::Item: AsRef<[u8]>;

    /// Delete a key-value pair.
    fn delete(&self, key: impl AsRef<[u8]>) -> Result<()>;

    /// Check if a key exists.
    fn contains(&self, key: impl AsRef<[u8]>) -> Result<bool>;
}

pub trait DatabaseCommonClear: DatabaseCommon {
    /// Clear the database.
    fn clear(&self) -> Result<()>;
}

/// Database column trait that returns references.
pub trait DatabaseCommonRef<'c>: DatabaseCommon {
    /// The type of the 'column', this is a reference to a database.
    type Ref: AsRef<[u8]> + 'c + std::ops::Deref<Target = [u8]> + Send + Sync;

    /// Get a value by key.
    fn get_ref(&'c self, key: impl AsRef<[u8]>) -> Result<Option<Self::Ref>>;
}

/// Database column trait that returns references in batch.
pub trait DatabaseCommonRefBatch<'c>: DatabaseCommon {
    /// The type of the 'column', this is a reference to a database.
    type Ref: AsRef<[u8]> + 'c + std::ops::Deref<Target = [u8]> + Send + Sync;

    /// Get a value by key in batch.
    fn get_multi_ref<I>(&self, keys: I) -> Result<Vec<Option<Self::Ref>>>
    where
        I: IntoIterator,
        I::Item: AsRef<[u8]>;
}

/// Database transaction trait.
pub trait DatabaseTxn: DatabaseCommon {
    /// Commit the transaction.
    fn commit(self) -> Result<()>;

    /// Rollback the transaction.
    fn rollback(self) -> Result<()>;
}

/// Database transaction trait that returns references.
pub trait DatabaseColumnTxn<'c>: DatabaseCommon {
    type Txn: DatabaseTxn;

    /// Start a transaction.
    fn transaction(&self) -> Result<Self::Txn>;
}
