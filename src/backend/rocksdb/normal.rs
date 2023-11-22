use std::sync::Arc;

use super::{BoundCFHandle, RocksDbImpl};
use crate::{backend::DBColumnRefBatch, Flushable, Innerable, Result};
use inherent::inherent;
use rocksdb::DBPinnableSlice;

/// A RocksDB database backend.
pub struct RocksDb {
    pub(crate) db: rocksdb::DB,
}

impl Flushable for RocksDb {
    fn flush(&self) -> Result<()> {
        self.db.flush()?;
        Ok(())
    }
}

/// A RocksDB database column family.
pub struct RocksDbColumn<'a> {
    pub(crate) name: String,
    pub(crate) env: &'a RocksDb,
    cf_handle: BoundCFHandle<'a>,
}

impl<'a> RocksDbColumn<'a> {
    pub(super) fn new(
        name: String,
        env: &'a RocksDb,
        cf_handle: Arc<rocksdb::BoundColumnFamily<'a>>,
    ) -> Self {
        Self {
            name,
            env,
            cf_handle: BoundCFHandle(cf_handle),
        }
    }
}

impl<'b> Innerable for RocksDbColumn<'b> {
    type Inner = Arc<rocksdb::BoundColumnFamily<'b>>;

    fn inner(&self) -> &Self::Inner {
        self.cf_handle.inner()
    }
}

impl Flushable for RocksDbColumn<'_> {
    fn flush(&self) -> Result<()> {
        self.env.db.flush_cf(self.inner())?;
        Ok(())
    }
}

#[inherent]
impl RocksDbImpl for RocksDb {
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
        Ok(Self { db })
    }
}

impl<'b, 'c> DBColumnRefBatch<'c> for RocksDbColumn<'b>
where
    'b: 'c,
{
    type Ref = DBPinnableSlice<'c>;

    fn get_multi_ref<I>(&self, keys: I) -> Result<Vec<Option<Self::Ref>>>
    where
        I: IntoIterator,
        I::Item: AsRef<[u8]>,
    {
        let values = self.env.db.batched_multi_get_cf(self.inner(), keys, false);
        let values = values
            .into_iter()
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(values)
    }
}
