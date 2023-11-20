// TODO: maybe use https://crates.io/crates/enum_dispatch instead of this?

use std::borrow::Cow;

use crate::Result;

use super::{DatabaseBackend, DatabaseColumn, DatabaseColumnRef};

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

impl<'a, 'c> DatabaseColumn<'c> for AnyDatabaseColumn<'a> {
    type Inner = AnyDatabaseBackendColumn<'a>;
    fn inner(&self) -> &Self::Inner {
        &self.column
    }

    fn set(&self, key: Cow<[u8]>, val: &[u8]) -> Result<()> {
        dispatch!(self, set, key, val)
    }

    fn get(&self, key: Cow<[u8]>) -> Result<Option<Vec<u8>>> {
        dispatch!(self, get, key)
    }
}
