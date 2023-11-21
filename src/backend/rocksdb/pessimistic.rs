use std::sync::Arc;

use super::RocksDbImpl;
use crate::{Innerable, Result};
use inherent::inherent;

/// A RocksDB database backend with pessimistic transactions.
pub struct RocksDbPessimistic<'a> {
    pub(crate) db: rocksdb::TransactionDB,
    marker: std::marker::PhantomData<&'a ()>,
}

/// A RocksDB database column family.
pub struct RocksDbPessimisticColumn<'a> {
    pub(crate) _name: String,
    pub(crate) _env: &'a RocksDbPessimistic<'a>,
    pub(crate) cf_handle: Arc<rocksdb::BoundColumnFamily<'a>>,
}

impl<'a> Innerable for RocksDbPessimisticColumn<'a> {
    type Inner = Arc<rocksdb::BoundColumnFamily<'a>>;
    fn inner(&self) -> &Self::Inner {
        &self.cf_handle
    }
}

#[inherent]
impl RocksDbImpl for RocksDbPessimistic<'_> {
    type RocksdbOptions = (rocksdb::Options, rocksdb::TransactionDBOptions);
    pub fn new(connect_str: &str) -> Result<Self>;
    pub fn list_databases(connect_str: &str) -> Result<Option<Vec<String>>>;

    pub fn new_with_config(
        mut config: (rocksdb::Options, rocksdb::TransactionDBOptions),
        connect_str: &str,
        cfs: &[String],
    ) -> Result<Self> {
        config.0.create_if_missing(true);
        let db = rocksdb::TransactionDB::open_cf(&config.0, &config.1, connect_str, cfs)?;
        Ok(Self {
            db,
            marker: Default::default(),
        })
    }
}
