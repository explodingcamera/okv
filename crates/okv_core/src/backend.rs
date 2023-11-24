use crate::{
    env::Env,
    error::Result,
    traits::{DBIterator, Innerable},
};

/// Database backend trait.
pub trait DatabaseBackend: Innerable + Sized + Send + Sync {
    /// The type of the 'column', this is a reference to a database.
    type Column: DBColumn;

    /// Create or open a database.
    fn create_or_open(env: Env<Self>, db: &str) -> Result<Self::Column>;
}

/// Database column trait.
pub trait DBColumn {
    /// Set a key-value pair.
    fn set(&self, key: impl AsRef<[u8]>, val: impl AsRef<[u8]>) -> Result<()>;

    /// Set a key-value pair if the key does not exist.
    fn set_nx(&self, key: impl AsRef<[u8]>, val: impl AsRef<[u8]>) -> Result<bool> {
        let key = key.as_ref();
        let val = val.as_ref();
        if self.contains(key)? {
            Ok(false)
        } else {
            self.set(key, val)?;
            Ok(true)
        }
    }

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
    fn get_multi_ref<I>(&'c self, keys: I) -> Result<Vec<Option<Self::Ref>>>
    where
        I: IntoIterator,
        I::Item: AsRef<[u8]>;
}

/// Database transaction trait that returns references.
pub trait DBColumnTransaction<'c>: DBColumn {
    type Txn: DBTransaction;

    /// Start a transaction.
    fn transaction(&'c self) -> Result<Self::Txn>;
}

/// Database transaction trait.
pub trait DBTransaction: DBColumn {
    /// Commit the transaction.
    fn commit(self) -> Result<()>;

    /// Rollback the transaction.
    fn rollback(self) -> Result<()>;
}

// FUTURE: use impl Trait when it's stable (https://github.com/rust-lang/rust/pull/115822)
/// Database Iterator trait.
pub trait DBColumnIterator {
    /// Create a new iterator.
    fn iter(&self) -> Result<DBIterator<Vec<u8>, Vec<u8>>>;
}

// FUTURE: use impl Trait when it's stable (https://github.com/rust-lang/rust/pull/115822)
/// Database Prefix Iterator trait.
pub trait DBColumnIteratorPrefix {
    /// Create a new iterator.
    fn iter_prefix(&self, prefix: impl AsRef<[u8]>) -> Result<DBIterator<Vec<u8>, Vec<u8>>>;
}
