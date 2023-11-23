use super::{BoundCFHandle, RocksDbImpl};
use crate::{backend::DBColumnRefBatch, Env, Flushable, Innerable, Result};
use inherent::inherent;
use rocksdb::DBPinnableSlice;
use self_cell::self_cell;

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
pub struct RocksDbColumn {
    pub(crate) name: String,
    pub(super) inner: RocksDbColumnInner,
}

self_cell!(
    pub struct RocksDbColumnInner {
        owner: Env<RocksDb>,

        #[covariant]
        dependent: BoundCFHandle,
    }
);

impl<'b> Innerable for RocksDbColumn {
    type Inner = RocksDbColumnInner;

    fn inner(&self) -> &Self::Inner {
        &self.inner
    }
}

impl Flushable for RocksDbColumn {
    fn flush(&self) -> Result<()> {
        self.db().flush_cf(self.cf_handle())?;
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

// TODO: Implement DBColumnRefBatch for RocksDbColumn
// impl<'b, 'c> DBColumnRefBatch<'c> for RocksDbColumn
// where
//     'b: 'c,
// {
//     type Ref = DBPinnableSlice<'c>;

//     fn get_multi_ref<I>(&self, keys: I) -> Result<Vec<Option<Self::Ref>>>
//     where
//         I: IntoIterator,
//         I::Item: AsRef<[u8]>,
//     {
//         let values = self
//             .db()
//             .batched_multi_get_cf(self.cf_handle(), keys, false);
//         let values = values
//             .into_iter()
//             .collect::<std::result::Result<Vec<_>, _>>()?;

//         Ok(values)
//     }
// }
