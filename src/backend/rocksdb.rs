use std::{borrow::Cow, sync::Arc};

use rocksdb::DBPinnableSlice;

use crate::{Error, Result};

use super::{DatabaseBackend, DatabaseColumn, DatabaseColumnRef};

pub struct RocksDb<'a> {
    db: rocksdb::DB,
    marker: std::marker::PhantomData<&'a ()>,
}

impl RocksDb<'_> {
    pub fn new(connect_str: &str) -> Result<Self> {
        let mut opts = rocksdb::Options::default();
        opts.create_if_missing(true);

        let cfs = rocksdb::DB::list_cf(&opts, connect_str)?;
        Ok(Self::new_with_config(opts, connect_str, &cfs)?)
    }

    pub fn list_databases(connect_str: &str) -> Result<Vec<String>> {
        Ok(rocksdb::DB::list_cf(
            &rocksdb::Options::default(),
            connect_str,
        )?)
    }

    // Create a new RocksDb instance with a custom configuration.
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
    type Column = RocksDbColumn<'c>;

    fn create_or_open(&'b self, name: &str) -> super::Result<Self::Column> {
        match self.db.cf_handle(name) {
            Some(handle) => {
                println!("Column family {} already exists", name);

                return Ok(RocksDbColumn {
                    _name: name.to_owned(),
                    _env: self,
                    cf_handle: handle,
                });
            }
            _ => {}
        };
        panic!("Column family {} already exists", name);

        println!("Creating column family {}", name);

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

pub struct RocksDbColumn<'a> {
    _name: String,
    _env: &'a RocksDb<'a>,
    cf_handle: Arc<rocksdb::BoundColumnFamily<'a>>,
}

impl<'b, 'c> DatabaseColumn<'c> for RocksDbColumn<'b> {
    fn set(&self, key: Cow<[u8]>, val: &[u8]) -> Result<()> {
        self._env.db.put_cf(&self.cf_handle, key, val)?;
        Ok(())
    }

    fn get(&self, key: Cow<[u8]>) -> Result<Option<Vec<u8>>> {
        match self._env.db.get_cf(&self.cf_handle, key)? {
            Some(x) => Ok(Some(x.to_vec())),
            None => Ok(None),
        }
    }
}

impl<'b, 'c> DatabaseColumnRef<'c> for RocksDbColumn<'b>
where
    'b: 'c,
{
    type Ref = DBPinnableSlice<'c>;

    fn get_ref(&self, key: Cow<[u8]>) -> Result<Option<Self::Ref>> {
        let x = self._env.db.get_pinned_cf(&self.cf_handle, key)?;
        let Some(x) = x else {
            return Ok(None);
        };
        Ok(Some(x))
    }
}