// TODO: maybe use https://crates.io/crates/enum_dispatch instead of this?

use super::{DBColumn, DBColumnClear, DatabaseBackend, Innerable};
use crate::Result;

/// Any Database Backend
pub enum AnyDatabaseBackend<'c> {
    #[cfg(feature = "memdb")]
    /// See [`MemDB`](super::mem::MemDB)
    MemDB(super::mem::MemDB<'c>),
    #[cfg(feature = "rocksdb")]
    /// See [`RocksDb`](super::rocksdb::RocksDb)
    RocksDB(super::rocksdb::RocksDb),
}

/// Any Database Column
pub enum AnyDatabaseBackendColumn<'c> {
    #[cfg(feature = "memdb")]
    MemDB(super::mem::MemDBColumn<'c>),
    #[cfg(feature = "rocksdb")]
    RocksDB(super::rocksdb::RocksDbColumn<'c>),
}

macro_rules! dispatch {
    ($self:ident, $func:ident, $($args:expr),*) => {
        match $self.column {
            #[cfg(feature = "memdb")]
            AnyDatabaseBackendColumn::MemDB(ref col) => col.$func($($args),*),
            #[cfg(feature = "rocksdb")]
            AnyDatabaseBackendColumn::RocksDB(ref col) => col.$func($($args),*),
        }
    };
    ($self:ident, $func:ident) => {
        match $self.column {
            #[cfg(feature = "memdb")]
            AnyDatabaseBackendColumn::MemDB(ref col) => col.$func(),
            #[cfg(feature = "rocksdb")]
            AnyDatabaseBackendColumn::RocksDB(ref col) => col.$func(),
        }
    };
}

/// Convenience wrapper for any database backend.
/// This can useful for testing and prototyping.
/// Not recommended for production use.
/// (Requires `unstable_any` feature)
pub struct AnyDatabase<'c> {
    marker: std::marker::PhantomData<&'c ()>,
    backend: AnyDatabaseBackend<'c>,
}

impl<'c> AnyDatabase<'c> {
    /// Create a new AnyDatabase wrapper.
    pub fn new(backend: AnyDatabaseBackend<'c>) -> Self {
        Self {
            marker: std::marker::PhantomData,
            backend,
        }
    }
}

impl<'a> Innerable for AnyDatabase<'a> {
    type Inner = AnyDatabaseBackend<'a>;
    fn inner(&self) -> &Self::Inner {
        &self.backend
    }
}

impl<'a> DatabaseBackend<'a> for AnyDatabase<'a> {
    type Column = AnyDatabaseColumn<'a>;

    fn create_or_open(&'a self, name: &str) -> Result<Self::Column> {
        let res = match self.backend {
            #[cfg(feature = "memdb")]
            AnyDatabaseBackend::MemDB(ref db) => {
                AnyDatabaseBackendColumn::MemDB(db.create_or_open(name)?)
            }
            #[cfg(feature = "rocksdb")]
            AnyDatabaseBackend::RocksDB(ref db) => {
                AnyDatabaseBackendColumn::RocksDB(db.create_or_open(name)?)
            }
        };

        Ok(AnyDatabaseColumn {
            marker: std::marker::PhantomData,
            column: res,
        })
    }
}

pub struct AnyDatabaseColumn<'c> {
    marker: std::marker::PhantomData<&'c ()>,
    column: AnyDatabaseBackendColumn<'c>,
}

impl<'a> Innerable for AnyDatabaseColumn<'a> {
    type Inner = AnyDatabaseBackendColumn<'a>;
    fn inner(&self) -> &Self::Inner {
        &self.column
    }
}

impl<'a> DBColumn for AnyDatabaseColumn<'a> {
    fn set(&self, key: impl AsRef<[u8]>, val: impl AsRef<[u8]>) -> Result<()> {
        dispatch!(self, set, key, val)
    }

    fn get(&self, key: impl AsRef<[u8]>) -> Result<Option<Vec<u8>>> {
        dispatch!(self, get, key)
    }

    fn get_multi<I>(&self, keys: I) -> Result<Vec<Option<Vec<u8>>>>
    where
        I: IntoIterator,
        I::Item: AsRef<[u8]>,
    {
        dispatch!(self, get_multi, keys)
    }

    fn contains(&self, key: impl AsRef<[u8]>) -> Result<bool> {
        dispatch!(self, contains, key)
    }

    fn delete(&self, key: impl AsRef<[u8]>) -> Result<()> {
        dispatch!(self, delete, key)
    }
}

impl<'a> DBColumnClear for AnyDatabaseColumn<'a> {
    fn clear(&self) -> Result<()> {
        match self.column {
            #[cfg(feature = "memdb")]
            AnyDatabaseBackendColumn::MemDB(ref col) => col.clear(),
            #[cfg(feature = "rocksdb")]
            AnyDatabaseBackendColumn::RocksDB(_) => unimplemented!("TODO: implement clear for rocksdb (might require a mutable reference to the database)"),
        }
    }
}
