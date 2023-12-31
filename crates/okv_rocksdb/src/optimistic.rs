use super::{BoundCFHandle, RocksDbImpl};
use crate::{okv_err, Env, Flushable, Result};
use inherent::inherent;
use self_cell::self_cell;

/// A RocksDB database backend with optimistic transactions.
pub struct RocksDbOptimistic {
    pub(crate) db: rocksdb::OptimisticTransactionDB,
}

impl Flushable for RocksDbOptimistic {
    fn flush(&self) -> Result<()> {
        self.db.flush().map_err(okv_err)?;
        Ok(())
    }
}

/// A RocksDB database column family.
pub struct RocksDbOptimisticColumn {
    pub(crate) name: String,
    pub(crate) inner: RocksDbOptimisticColumnInner,
}

self_cell!(
    /// A RocksDB database column family.
    pub(crate) struct RocksDbOptimisticColumnInner {
        owner: Env<RocksDbOptimistic>,

        #[covariant]
        dependent: BoundCFHandle,
    }
);

#[inherent]
impl RocksDbImpl for RocksDbOptimistic {
    type RocksdbOptions = rocksdb::Options;
    pub fn new(connect_str: &str) -> Result<Self, rocksdb::Error>;
    pub fn list_databases(connect_str: &str) -> Result<Option<Vec<String>>, rocksdb::Error>;

    pub fn new_with_config(
        mut config: rocksdb::Options,
        connect_str: &str,
        cfs: &[String],
    ) -> Result<Self, rocksdb::Error> {
        config.create_if_missing(true);
        let db = rocksdb::OptimisticTransactionDB::open_cf(&config, connect_str, cfs)?;
        Ok(Self { db })
    }
}
