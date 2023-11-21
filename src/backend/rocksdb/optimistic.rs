use std::sync::Arc;

use super::RocksDbImpl;
use crate::{
    backend::{Flushable, Innerable},
    Result,
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
    pub(crate) _name: String,
    pub(crate) _env: &'a RocksDbOptimistic<'a>,
    pub(crate) cf_handle: Arc<rocksdb::BoundColumnFamily<'a>>,
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
