use std::sync::Arc;

use rocksdb::DBPinnableSlice;

use crate::{Error, Result};

use super::{DatabaseBackend, DatabaseColumn, DatabaseColumnRef, Flushable};

/// A RocksDB database backend.
pub struct RocksDb<'a> {
    db: rocksdb::DB,
    marker: std::marker::PhantomData<&'a ()>,
}

impl RocksDb<'_> {
    /// Create a new RocksDb instance with default configuration.
    /// Automatically creates the database if it doesn't exist.
    pub fn new(connect_str: &str) -> Result<Self> {
        let cfs = Self::list_databases(connect_str)?.unwrap_or(vec![]);
        let mut opts = rocksdb::Options::default();
        opts.create_if_missing(true);
        Self::new_with_config(opts, connect_str, &cfs)
    }

    /// List all databases (column families) in a RocksDb instance.
    /// Returns `None` if there was an io error (e.g. the database doesn't exist)
    pub fn list_databases(connect_str: &str) -> Result<Option<Vec<String>>> {
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
    pub fn new_with_config(
        config: rocksdb::Options,
        connect_str: &str,
        cfs: &[String],
    ) -> Result<Self> {
        let db = rocksdb::DB::open_cf(&config, connect_str, cfs)?;
        Ok(Self {
            db,
            marker: std::marker::PhantomData,
        })
    }
}

impl<'b, 'c> DatabaseBackend<'b, 'c> for RocksDb<'b>
where
    'b: 'c,
{
    type Inner = rocksdb::DB;
    fn inner(&self) -> &Self::Inner {
        &self.db
    }

    type Column = RocksDbColumn<'c>;
    fn create_or_open(&'b self, name: &str) -> super::Result<Self::Column> {
        if let Some(handle) = self.db.cf_handle(name) {
            return Ok(RocksDbColumn {
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

        Ok(RocksDbColumn {
            _name: name.to_owned(),
            _env: self,
            cf_handle: handle,
        })
    }
}

impl Flushable for RocksDb<'_> {
    fn flush(&self) -> Result<()> {
        self.db.flush()?;
        Ok(())
    }
}

/// A RocksDB database column family.
pub struct RocksDbColumn<'a> {
    _name: String,
    _env: &'a RocksDb<'a>,
    cf_handle: Arc<rocksdb::BoundColumnFamily<'a>>,
}

impl<'b> DatabaseColumn for RocksDbColumn<'b> {
    type Inner = Arc<rocksdb::BoundColumnFamily<'b>>;

    fn inner(&self) -> &Self::Inner {
        &self.cf_handle
    }

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

    fn clear(&self) -> Result<()> {
        self._env.db.drop_cf(&self._name)?;
        self._env
            .db
            .create_cf(&self._name, &rocksdb::Options::default())?;

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

impl<'b, 'c> DatabaseColumnRef<'c> for RocksDbColumn<'b>
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

    fn get_multi_ref<I>(&self, keys: I) -> Result<Vec<Option<Self::Ref>>>
    where
        I: IntoIterator,
        I::Item: AsRef<[u8]>,
    {
        let values = self
            ._env
            .db
            .batched_multi_get_cf(&self.cf_handle, keys, false);

        let values = values
            .into_iter()
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(values)
    }
}

impl Flushable for RocksDbColumn<'_> {
    fn flush(&self) -> Result<()> {
        self._env.db.flush_cf(&self.cf_handle)?;
        Ok(())
    }
}
