use super::{BoundCFHandle, RocksDbImpl};
use crate::{Env, Result};
use inherent::inherent;
use self_cell::self_cell;

/// A RocksDB database backend with pessimistic transactions.
pub struct RocksDbPessimistic {
    pub(crate) db: rocksdb::TransactionDB,
}

/// A RocksDB database column family.
pub struct RocksDbPessimisticColumn {
    pub(crate) name: String,
    pub(crate) inner: RocksDbPessimisticColumnInner,
}

self_cell!(
    /// A RocksDB database column family.
    pub(crate) struct RocksDbPessimisticColumnInner {
        owner: Env<RocksDbPessimistic>,

        #[covariant]
        dependent: BoundCFHandle,
    }
);

#[inherent]
impl RocksDbImpl for RocksDbPessimistic {
    type RocksdbOptions = (rocksdb::Options, rocksdb::TransactionDBOptions);
    pub fn new(connect_str: &str) -> Result<Self, rocksdb::Error>;
    pub fn list_databases(connect_str: &str) -> Result<Option<Vec<String>>, rocksdb::Error>;

    pub fn new_with_config(
        mut config: (rocksdb::Options, rocksdb::TransactionDBOptions),
        connect_str: &str,
        cfs: &[String],
    ) -> Result<Self, rocksdb::Error> {
        config.0.create_if_missing(true);
        let db = rocksdb::TransactionDB::open_cf(&config.0, &config.1, connect_str, cfs)?;
        Ok(Self { db })
    }
}
