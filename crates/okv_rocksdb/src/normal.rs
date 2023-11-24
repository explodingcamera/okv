use crate::okv_err;

use super::{BoundCFHandle, RocksDbImpl};
use inherent::inherent;
use okv_core::{backend::DBColumnRefBatch, env::Env, error::Result, traits::Flushable};
use rocksdb::DBPinnableSlice;
use self_cell::self_cell;

/// A RocksDB database backend.
pub struct RocksDb {
    pub(crate) db: rocksdb::DB,
}

impl Flushable for RocksDb {
    fn flush(&self) -> Result<()> {
        self.db.flush().map_err(okv_err)?;
        Ok(())
    }
}

/// A RocksDB database column family.
pub struct RocksDbColumn {
    pub(crate) name: String,
    pub(crate) inner: RocksDbColumnInner,
}

self_cell!(
    /// A RocksDB database column family.
    pub(crate) struct RocksDbColumnInner {
        owner: Env<RocksDb>,

        #[covariant]
        dependent: BoundCFHandle,
    }
);

impl Flushable for RocksDbColumn {
    fn flush(&self) -> Result<()> {
        self.db().flush_cf(self.cf_handle()).map_err(okv_err)?;
        Ok(())
    }
}

#[inherent]
impl RocksDbImpl for RocksDb {
    type RocksdbOptions = rocksdb::Options;
    pub fn new(connect_str: &str) -> Result<Self, rocksdb::Error>;
    pub fn list_databases(connect_str: &str) -> Result<Option<Vec<String>>, rocksdb::Error>;

    pub fn new_with_config(
        mut config: rocksdb::Options,
        connect_str: &str,
        cfs: &[String],
    ) -> Result<Self, rocksdb::Error> {
        config.create_if_missing(true);
        let db = rocksdb::DB::open_cf(&config, connect_str, cfs.iter())?;
        Ok(Self { db })
    }
}

impl<'c> DBColumnRefBatch<'c> for RocksDbColumn {
    type Ref = DBPinnableSlice<'c>;

    fn get_multi_ref<I>(&'c self, keys: I) -> Result<Vec<Option<Self::Ref>>>
    where
        I: IntoIterator,
        I::Item: AsRef<[u8]>,
    {
        let values = self
            .db()
            .batched_multi_get_cf(self.cf_handle(), keys, false);
        let values = values
            .into_iter()
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(okv_err)?;

        Ok(values)
    }
}
