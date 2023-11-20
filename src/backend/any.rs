// TODO: maybe use https://crates.io/crates/enum_dispatch instead of this?

use super::{DatabaseBackend, DatabaseColumn};
use crate::Result;

pub enum AnyDatabaseBackend<'c> {
    #[cfg(feature = "memdb")]
    MemDB(super::mem::MemDB<'c>),
    #[cfg(feature = "rocksdb")]
    RocksDB(super::rocksdb::RocksDb<'c>),
}

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

pub struct AnyDatabase<'c> {
    marker: std::marker::PhantomData<&'c ()>,
    backend: AnyDatabaseBackend<'c>,
}

impl<'c> AnyDatabase<'c> {
    pub fn new(backend: AnyDatabaseBackend<'c>) -> Self {
        Self {
            marker: std::marker::PhantomData,
            backend,
        }
    }
}

impl<'b, 'c> DatabaseBackend<'b, 'c> for AnyDatabase<'b>
where
    'b: 'c,
    Self: Sized,
{
    type Inner = AnyDatabaseBackend<'c>;
    type Column = AnyDatabaseColumn<'c>;

    fn create_or_open(&'b self, name: &str) -> Result<Self::Column> {
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
    fn inner(&self) -> &Self::Inner {
        &self.backend
    }
}

pub struct AnyDatabaseColumn<'c> {
    marker: std::marker::PhantomData<&'c ()>,
    column: AnyDatabaseBackendColumn<'c>,
}

impl<'a> DatabaseColumn for AnyDatabaseColumn<'a> {
    type Inner = AnyDatabaseBackendColumn<'a>;
    fn inner(&self) -> &Self::Inner {
        &self.column
    }

    fn set(&self, key: impl AsRef<[u8]>, val: &[u8]) -> Result<()> {
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

    fn clear(&self) -> Result<()> {
        dispatch!(self, clear)
    }
}
