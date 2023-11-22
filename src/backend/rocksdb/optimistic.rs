use std::sync::Arc;

use super::{BoundCFHandle, RocksDbImpl};
use crate::{
    Result, {Flushable, Innerable},
};
use inherent::inherent;

/// A RocksDB database backend with optimistic transactions.
pub struct RocksDbOptimistic<'a> {
    pub(crate) db: rocksdb::OptimisticTransactionDB,
    marker: std::marker::PhantomData<&'a ()>,
}

impl Flushable for RocksDbOptimistic<'_> {
    fn flush(&self) -> Result<()> {
        self.db.flush()?;
        Ok(())
    }
}

/// A RocksDB database column family.
pub struct RocksDbOptimisticColumn<'a> {
    pub(crate) name: String,
    pub(crate) env: &'a RocksDbOptimistic<'a>,
    cf_handle: BoundCFHandle<'a>,
}

impl<'a> RocksDbOptimisticColumn<'a> {
    pub(super) fn new(
        name: String,
        env: &'a RocksDbOptimistic<'a>,
        cf_handle: Arc<rocksdb::BoundColumnFamily<'a>>,
    ) -> Self {
        Self {
            name,
            env,
            cf_handle: BoundCFHandle(cf_handle),
        }
    }
}

impl<'a> Innerable for RocksDbOptimisticColumn<'a> {
    type Inner = Arc<rocksdb::BoundColumnFamily<'a>>;

    fn inner(&self) -> &Self::Inner {
        self.cf_handle.inner()
    }
}

#[inherent]
impl RocksDbImpl for RocksDbOptimistic<'_> {
    type RocksdbOptions = rocksdb::Options;
    pub fn new(connect_str: &str) -> Result<Self>;
    pub fn list_databases(connect_str: &str) -> Result<Option<Vec<String>>>;

    pub fn new_with_config(
        mut config: rocksdb::Options,
        connect_str: &str,
        cfs: &[String],
    ) -> Result<Self> {
        config.create_if_missing(true);
        let db = rocksdb::OptimisticTransactionDB::open_cf(&config, connect_str, cfs)?;
        Ok(Self {
            db,
            marker: Default::default(),
        })
    }
}
