use std::sync::Arc;

use super::RocksDbImpl;
use crate::{
    backend::{DatabaseCommonRef, DatabaseCommonRefBatch},
    Flushable, Innerable, Result,
};
use inherent::inherent;
use rocksdb::DBPinnableSlice;

/// A RocksDB database backend.
pub struct RocksDb<'a> {
    pub(crate) db: rocksdb::DB,
    marker: std::marker::PhantomData<&'a ()>,
}

impl Flushable for RocksDb<'_> {
    fn flush(&self) -> Result<()> {
        self.db.flush()?;
        Ok(())
    }
}

/// A RocksDB database column family.
pub struct RocksDbColumn<'a> {
    pub(crate) _name: String,
    pub(crate) _env: &'a RocksDb<'a>,
    pub(crate) cf_handle: Arc<rocksdb::BoundColumnFamily<'a>>,
}

impl<'b> Innerable for RocksDbColumn<'b> {
    type Inner = Arc<rocksdb::BoundColumnFamily<'b>>;

    fn inner(&self) -> &Self::Inner {
        &self.cf_handle
    }
}

impl Flushable for RocksDbColumn<'_> {
    fn flush(&self) -> Result<()> {
        self._env.db.flush_cf(&self.cf_handle)?;
        Ok(())
    }
}

#[inherent]
impl RocksDbImpl for RocksDb<'_> {
    type RocksdbOptions = rocksdb::Options;
    pub fn new(connect_str: &str) -> Result<Self>;
    pub fn list_databases(connect_str: &str) -> Result<Option<Vec<String>>>;

    pub fn new_with_config(
        mut config: rocksdb::Options,
        connect_str: &str,
        cfs: &[String],
    ) -> Result<Self> {
        config.create_if_missing(true);
        let db = rocksdb::DB::open_cf(&config, connect_str, cfs.iter())?;
        Ok(Self {
            db,
            marker: Default::default(),
        })
    }
}

impl<'b, 'c> DatabaseCommonRefBatch<'c> for RocksDbColumn<'b>
where
    'b: 'c,
{
    type Ref = DBPinnableSlice<'c>;

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
