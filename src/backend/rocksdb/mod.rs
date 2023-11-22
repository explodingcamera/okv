use super::{DBColumn, DBColumnDelete, DBColumnRef, DatabaseBackend};
use crate::{Error, Innerable, Result};
use rocksdb::{DBPinnableSlice, OptimisticTransactionDB, TransactionDB, DB};
use std::sync::Arc;

mod normal;
mod optimistic;
mod pessimistic;

mod tx;

pub use normal::*;
pub use optimistic::*;
pub use pessimistic::*;

pub(super) struct BoundCFHandle<'a>(Arc<rocksdb::BoundColumnFamily<'a>>);

impl<'a> Innerable for BoundCFHandle<'a> {
    type Inner = Arc<rocksdb::BoundColumnFamily<'a>>;
    fn inner(&self) -> &Self::Inner {
        &self.0
    }
}

// Safety: see https://github.com/rust-rocksdb/rust-rocksdb/issues/407
// I'm not sure if this is the best way to do this, but it seems to work.
// Using a Mutex is really ugly, since RocksDB expects a Arc<DB> and not a Arc<Mutex<DB>> => https://github.com/rust-rocksdb/rust-rocksdb/issues/803
unsafe impl Sync for BoundCFHandle<'_> {}
unsafe impl Send for BoundCFHandle<'_> {}

trait RocksDbImpl: Sized {
    type RocksdbOptions: Default;

    /// Create a new RocksDb instance with default configuration.
    /// Automatically creates the database if it doesn't exist.
    fn new(connect_str: &str) -> Result<Self> {
        let cfs = Self::list_databases(connect_str)?.unwrap_or(vec![]);
        let opts = Self::RocksdbOptions::default();
        Self::new_with_config(opts, connect_str, &cfs)
    }

    /// List all databases (column families) in a RocksDb instance.
    /// Returns `None` if there was an io error (e.g. the database doesn't exist)
    fn list_databases(connect_str: &str) -> Result<Option<Vec<String>>> {
        let cfs = match rocksdb::DB::list_cf(&rocksdb::Options::default(), connect_str) {
            Err(e) => {
                println!("Error: {:?}", e.kind());
                if e.kind() != rocksdb::ErrorKind::IOError {
                    return Ok(None);
                }
                Some(vec![])
            }
            Ok(cfs) => Some(cfs),
        };

        Ok(cfs)
    }

    /// Create a new RocksDb instance with a custom configuration.
    /// Note that rocksdb requires that all databases (column families) are opened at startup.
    /// To get all column families, use `list_databases`.
    fn new_with_config(
        config: Self::RocksdbOptions,
        connect_str: &str,
        cfs: &[String],
    ) -> Result<Self>;
}

macro_rules! implement_column {
    ($name:ident) => {
        impl<'a> DBColumnDelete for $name<'a> {
            fn delete_db(&self) -> Result<()> {
                self.env.db.drop_cf(&self.name)?;
                Ok(())
            }
        }

        impl<'b, 'c> DBColumnRef<'c> for $name<'b>
        where
            'b: 'c,
        {
            type Ref = DBPinnableSlice<'c>;

            fn get_ref(&self, key: impl AsRef<[u8]>) -> Result<Option<Self::Ref>> {
                let x = self.env.db.get_pinned_cf(self.inner().into(), key)?;
                let Some(x) = x else {
                    return Ok(None);
                };
                Ok(Some(x))
            }
        }

        impl<'a> DBColumn for $name<'a> {
            fn set(&self, key: impl AsRef<[u8]>, val: impl AsRef<[u8]>) -> Result<()> {
                self.env.db.put_cf(self.inner(), key, val)?;
                Ok(())
            }

            fn get(&self, key: impl AsRef<[u8]>) -> Result<Option<Vec<u8>>> {
                match self.env.db.get_cf(self.inner(), key)? {
                    Some(x) => Ok(Some(x.to_vec())),
                    None => Ok(None),
                }
            }

            fn contains(&self, key: impl AsRef<[u8]>) -> Result<bool> {
                match self.env.db.get_cf(self.inner(), key)? {
                    Some(_) => Ok(true),
                    None => Ok(false),
                }
            }

            fn delete(&self, key: impl AsRef<[u8]>) -> Result<()> {
                self.env.db.delete_cf(self.inner(), key)?;
                Ok(())
            }

            fn get_multi<I>(&self, keys: I) -> Result<Vec<Option<Vec<u8>>>>
            where
                I: IntoIterator,
                I::Item: AsRef<[u8]>,
            {
                let keys = keys.into_iter().map(|key| (self.inner(), key));
                let values = self.env.db.multi_get_cf(keys);
                let values = values
                    .into_iter()
                    .collect::<std::result::Result<Vec<_>, _>>()?;
                Ok(values)
            }
        }
    };
}

macro_rules! implement_backend {
    ($name:ident, $col:ident, $db:ident) => {
        impl DatabaseBackend for $name {
            type Column<'c> = $col<'c> where Self: 'c;
            fn create_or_open<'c>(&'c self, name: &str) -> super::Result<Self::Column<'c>> {
                if let Some(handle) = self.db.cf_handle(name) {
                    return Ok($col::new(name.to_owned(), self, handle));
                };

                let cf_opts = rocksdb::Options::default();
                self.db.create_cf(name, &cf_opts)?;
                let handle = self.db.cf_handle(name).ok_or(Error::DatabaseNotFound {
                    db: name.to_string(),
                })?;

                Ok($col::new(name.to_owned(), self, handle))
            }
        }

        impl<'a> crate::backend::Innerable for $name {
            type Inner = $db;
            fn inner(&self) -> &Self::Inner {
                &self.db
            }
        }
    };
}

implement_column!(RocksDbOptimisticColumn);
implement_column!(RocksDbPessimisticColumn);
implement_column!(RocksDbColumn);

implement_backend!(
    RocksDbOptimistic,
    RocksDbOptimisticColumn,
    OptimisticTransactionDB
);
implement_backend!(RocksDbPessimistic, RocksDbPessimisticColumn, TransactionDB);
implement_backend!(RocksDb, RocksDbColumn, DB);
