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
    type Column: DBColumn;

    /// Create or open a database.
    fn create_or_open(&'a self, db: &str) -> Result<Self::Column>;
}

/// Database column trait.
pub trait DBColumn {
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

pub trait DBColumnClear: DBColumn {
    /// Clear the database.
    fn clear(&self) -> Result<()>;
}

pub trait DBColumnDelete: DBColumn {
    /// Delete the database. Note that this will delete all data in the database.
    /// After calling this method, the database should not be used anymore or it
    /// will panic.
    fn delete_db(&self) -> Result<()>;
}

/// Database column trait that returns references.
pub trait DBColumnRef<'c>: DBColumn {
    /// The type of the 'column', this is a reference to a database.
    type Ref: AsRef<[u8]> + 'c + std::ops::Deref<Target = [u8]> + Send + Sync;

    /// Get a value by key.
    fn get_ref(&'c self, key: impl AsRef<[u8]>) -> Result<Option<Self::Ref>>;
}

/// Database column trait that returns references in batch.
pub trait DBColumnRefBatch<'c>: DBColumn {
    /// The type of the 'column', this is a reference to a database.
    type Ref: AsRef<[u8]> + 'c + std::ops::Deref<Target = [u8]> + Send + Sync;

    /// Get a value by key in batch.
    fn get_multi_ref<I>(&self, keys: I) -> Result<Vec<Option<Self::Ref>>>
    where
        I: IntoIterator,
        I::Item: AsRef<[u8]>;
}

/// Database transaction trait that returns references.
pub trait DBColumnTransaction<'c>: DBColumn {
    type Txn: DBTransaction;

    /// Start a transaction.
    fn transaction(&self) -> Result<Self::Txn>;
}

/// Database transaction trait.
pub trait DBTransaction: DBColumn {
    /// Commit the transaction.
    fn commit(self) -> Result<()>;

    /// Rollback the transaction.
    fn rollback(self) -> Result<()>;
}
