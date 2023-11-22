use super::{DatabaseBackend, DatabaseCommon, DatabaseCommonClear, DatabaseCommonRef};
use crate::{Error, Result};
use rocksdb::{DBPinnableSlice, OptimisticTransactionDB, TransactionDB, DB};

mod normal;
mod optimistic;
mod pessimistic;

mod tx;

pub use normal::*;
pub use optimistic::*;
pub use pessimistic::*;

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
        // impl<'a> DatabaseCommonClear for $name<'a> {
        //     fn clear(&self) -> Result<()> {
        //         unimplemented!("TODO: implement clear for rocksdb (might require a mutable reference to the database)");

        //         // self._env.db.drop_cf(&self._name)?;
        //         // self._env
        //         //     .db
        //         //     .create_cf(&self._name, &rocksdb::Options::default())?;
        //         // Ok(())
        //     }
        // }

        impl<'b, 'c> DatabaseCommonRef<'c> for $name<'b>
        where
            'b: 'c,
        {
            type Ref = DBPinnableSlice<'c>;

            fn get_ref(&self, key: impl AsRef<[u8]>) -> Result<Option<Self::Ref>> {
                let x = self._env.db.get_pinned_cf(&self.cf_handle, key)?;
                let Some(x) = x else {
                    return Ok(None);
                };
                Ok(Some(x))
            }
        }

        impl<'a> DatabaseCommon for $name<'a> {
            fn set(&self, key: impl AsRef<[u8]>, val: &[u8]) -> Result<()> {
                self._env.db.put_cf(&self.cf_handle, key, val)?;
                Ok(())
            }

            fn get(&self, key: impl AsRef<[u8]>) -> Result<Option<Vec<u8>>> {
                match self._env.db.get_cf(&self.cf_handle, key)? {
                    Some(x) => Ok(Some(x.to_vec())),
                    None => Ok(None),
                }
            }

            fn contains(&self, key: impl AsRef<[u8]>) -> Result<bool> {
                match self._env.db.get_cf(&self.cf_handle, key)? {
                    Some(_) => Ok(true),
                    None => Ok(false),
                }
            }

            fn delete(&self, key: impl AsRef<[u8]>) -> Result<()> {
                self._env.db.delete_cf(&self.cf_handle, key)?;
                Ok(())
            }

            fn get_multi<I>(&self, keys: I) -> Result<Vec<Option<Vec<u8>>>>
            where
                I: IntoIterator,
                I::Item: AsRef<[u8]>,
            {
                let keys = keys.into_iter().map(|key| (&self.cf_handle, key));
                let values = self._env.db.multi_get_cf(keys);
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
        impl<'a> DatabaseBackend<'a> for $name<'a> {
            type Column = $col<'a>;
            fn create_or_open(&'a self, name: &str) -> super::Result<Self::Column> {
                if let Some(handle) = self.db.cf_handle(name) {
                    return Ok($col {
                        _name: name.to_owned(),
                        _env: self,
                        cf_handle: handle,
                    });
                };

                let cf_opts = rocksdb::Options::default();
                self.db.create_cf(name, &cf_opts)?;
                let handle = self.db.cf_handle(name).ok_or(Error::DatabaseNotFound {
                    db: name.to_string(),
                })?;

                Ok($col {
                    _name: name.to_owned(),
                    _env: self,
                    cf_handle: handle,
                })
            }
        }

        impl<'a> crate::backend::Innerable for $name<'a> {
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
