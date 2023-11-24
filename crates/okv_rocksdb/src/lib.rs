use okv_core::{backend::*, env::Env, error::*, traits::*};
use rocksdb::{BoundColumnFamily, DBPinnableSlice, OptimisticTransactionDB, TransactionDB, DB};
use std::sync::Arc;

mod normal;
mod optimistic;
mod pessimistic;

mod tx;
pub use normal::*;
pub use optimistic::*;
pub use pessimistic::*;

pub(crate) fn okv_err(e: rocksdb::Error) -> Error {
    Error::DatabaseBackend(Box::new(e))
}

/// A bound column family handle for RocksDB.
pub struct BoundCFHandle<'a>(Arc<rocksdb::BoundColumnFamily<'a>>);

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
    fn new(connect_str: &str) -> Result<Self, rocksdb::Error> {
        let cfs = Self::list_databases(connect_str)?.unwrap_or(vec![]);
        let opts = Self::RocksdbOptions::default();
        Self::new_with_config(opts, connect_str, &cfs)
    }

    /// List all databases (column families) in a RocksDb instance.
    /// Returns `None` if there was an io error (e.g. the database doesn't exist)
    fn list_databases(connect_str: &str) -> Result<Option<Vec<String>>, rocksdb::Error> {
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
    ) -> Result<Self, rocksdb::Error>;
}

macro_rules! implement_column_traits {
    ($name:ident) => {
        impl DBColumnDelete for $name {
            fn delete_db(&self) -> Result<()> {
                self.db().drop_cf(&self.name.clone()).map_err(okv_err)?;
                Ok(())
            }
        }

        impl<'c> DBColumnRef<'c> for $name {
            type Ref = DBPinnableSlice<'c>;

            fn get_ref(&'c self, key: impl AsRef<[u8]>) -> Result<Option<Self::Ref>> {
                let x = self
                    .db()
                    .get_pinned_cf(self.cf_handle(), key)
                    .map_err(okv_err)?;
                let Some(x) = x else {
                    return Ok(None);
                };

                Ok(Some(x))
            }
        }

        impl DBColumnIterator for $name {
            fn iter(&self) -> Result<Box<dyn Iterator<Item = Result<(Vec<u8>, Vec<u8>)>> + '_>> {
                let iter = self
                    .db()
                    .iterator_cf(self.cf_handle(), rocksdb::IteratorMode::Start)
                    .map(|v| match v {
                        Ok((k, v)) => Ok((k.to_vec(), v.to_vec())),
                        Err(e) => Err(okv_err(e)),
                    });

                Ok(Box::new(iter))
            }
        }

        impl DBColumnIteratorPrefix for $name {
            fn iter_prefix(
                &self,
                prefix: impl AsRef<[u8]>,
            ) -> Result<Box<dyn Iterator<Item = Result<(Vec<u8>, Vec<u8>)>> + '_>> {
                let iter = self
                    .db()
                    .iterator_cf(
                        self.cf_handle(),
                        rocksdb::IteratorMode::From(prefix.as_ref(), rocksdb::Direction::Forward),
                    )
                    .map(|v| match v {
                        Ok((k, v)) => Ok((k.to_vec(), v.to_vec())),
                        Err(e) => Err(okv_err(e)),
                    });

                Ok(Box::new(iter))
            }
        }

        impl DBColumn for $name {
            fn set(&self, key: impl AsRef<[u8]>, val: impl AsRef<[u8]>) -> Result<()> {
                self.db()
                    .put_cf(self.cf_handle(), key, val)
                    .map_err(okv_err)?;
                Ok(())
            }

            fn get(&self, key: impl AsRef<[u8]>) -> Result<Option<Vec<u8>>> {
                match self.db().get_cf(self.cf_handle(), key).map_err(okv_err)? {
                    Some(x) => Ok(Some(x.to_vec())),
                    None => Ok(None),
                }
            }

            fn contains(&self, key: impl AsRef<[u8]>) -> Result<bool> {
                match self.db().get_cf(self.cf_handle(), key).map_err(okv_err)? {
                    Some(_) => Ok(true),
                    None => Ok(false),
                }
            }

            fn delete(&self, key: impl AsRef<[u8]>) -> Result<()> {
                self.db()
                    .delete_cf(self.cf_handle(), key)
                    .map_err(okv_err)?;
                Ok(())
            }

            fn get_multi<I>(&self, keys: I) -> Result<Vec<Option<Vec<u8>>>>
            where
                I: IntoIterator,
                I::Item: AsRef<[u8]>,
            {
                let keys = keys.into_iter().map(|key| (self.cf_handle(), key));
                let values = self.db().multi_get_cf(keys);
                let values = values
                    .into_iter()
                    .collect::<std::result::Result<Vec<_>, _>>()
                    .map_err(okv_err)?;
                Ok(values)
            }
        }
    };
}

macro_rules! implement_column {
    ($name:ident, $col:ident, $col_inner:ident, $db:ident) => {
        impl $col {
            pub(crate) fn cf_handle(&self) -> &Arc<BoundColumnFamily<'_>> {
                self.inner.borrow_dependent().inner()
            }

            pub(crate) fn db(&self) -> &$db {
                &self.inner.borrow_owner().db().db
            }

            pub fn try_new(env: Env<$name>, name: String) -> Result<Self> {
                let inner = $col_inner::try_new(env, |env| {
                    let handle = if let Some(handle) = env.db().db.cf_handle(&name) {
                        handle
                    } else {
                        let cf_opts = rocksdb::Options::default();
                        env.db()
                            .db
                            .create_cf(name.clone(), &cf_opts)
                            .map_err(okv_err)?;
                        env.db()
                            .db
                            .cf_handle(&name)
                            .ok_or(Error::DatabaseNotFound {
                                db: name.to_string(),
                            })?
                    };

                    Ok::<BoundCFHandle<'_>, Error>(BoundCFHandle(handle))
                })?;

                Ok(Self { name, inner })
            }
        }
    };
}

macro_rules! implement_backend {
    ($name:ident, $col:ident, $db:ident) => {
        impl DatabaseBackend for $name {
            type Column = $col;
            fn create_or_open(env: Env<$name>, name: &str) -> Result<Self::Column> {
                $col::try_new(env, name.to_owned())
            }
        }

        impl<'a> Innerable for $name {
            type Inner = $db;
            fn inner(&self) -> &Self::Inner {
                &self.db
            }
        }
    };
}

implement_column_traits!(RocksDbColumn);
implement_column!(RocksDb, RocksDbColumn, RocksDbColumnInner, DB);
implement_backend!(RocksDb, RocksDbColumn, DB);

implement_column_traits!(RocksDbOptimisticColumn);
implement_backend!(
    RocksDbOptimistic,
    RocksDbOptimisticColumn,
    OptimisticTransactionDB
);
implement_column!(
    RocksDbOptimistic,
    RocksDbOptimisticColumn,
    RocksDbOptimisticColumnInner,
    OptimisticTransactionDB
);

implement_column_traits!(RocksDbPessimisticColumn);
implement_backend!(RocksDbPessimistic, RocksDbPessimisticColumn, TransactionDB);
implement_column!(
    RocksDbPessimistic,
    RocksDbPessimisticColumn,
    RocksDbPessimisticColumnInner,
    TransactionDB
);
