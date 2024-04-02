use std::sync::Arc;

use rocksdb::{DBPinnableSlice, OptimisticTransactionDB, TransactionDB};

use okv_core::backend::*;
use okv_core::error::Result;

use crate::okv_err;

use super::{RocksDbOptimisticColumn, RocksDbPessimisticColumn};

pub struct RocksDBTransaction<'a, DB> {
    pub(crate) cf_handle: Arc<rocksdb::BoundColumnFamily<'a>>,
    tx: rocksdb::Transaction<'a, DB>,
}

impl<'a> DBColumnTransaction<'a> for RocksDbOptimisticColumn {
    type Txn = RocksDBTransaction<'a, OptimisticTransactionDB>;

    fn transaction(&'a self) -> Result<Self::Txn> {
        let tx = self.db().transaction();
        Ok(RocksDBTransaction {
            tx,
            cf_handle: self.cf_handle().clone(),
        })
    }
}

impl<'a> DBColumnTransaction<'a> for RocksDbPessimisticColumn {
    type Txn = RocksDBTransaction<'a, TransactionDB>;

    fn transaction(&'a self) -> Result<Self::Txn> {
        let tx = self.db().transaction();
        Ok(RocksDBTransaction {
            tx,
            cf_handle: self.cf_handle().clone(),
        })
    }
}

impl<'a, DB> DBTransaction for RocksDBTransaction<'a, DB> {
    fn commit(self) -> Result<()> {
        self.tx.commit().map_err(okv_err)?;
        Ok(())
    }

    fn rollback(self) -> Result<()> {
        self.tx.rollback().map_err(okv_err)?;
        Ok(())
    }
}

impl<'a, DB> DBColumn for RocksDBTransaction<'a, DB> {
    fn set(&self, key: impl AsRef<[u8]>, val: impl AsRef<[u8]>) -> Result<()> {
        self.tx.put_cf(&self.cf_handle, key, val).map_err(okv_err)?;
        Ok(())
    }

    fn get(&self, key: impl AsRef<[u8]>) -> Result<Option<Vec<u8>>> {
        match self.tx.get_cf(&self.cf_handle, key).map_err(okv_err)? {
            Some(x) => Ok(Some(x.to_vec())),
            None => Ok(None),
        }
    }

    fn get_multi<I>(&self, keys: I) -> Result<Vec<Option<Vec<u8>>>>
    where
        I: IntoIterator,
        I::Item: AsRef<[u8]>,
    {
        let keys = keys.into_iter().map(|key| (&self.cf_handle, key));
        let values = self.tx.multi_get_cf(keys);
        let values = values
            .into_iter()
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(okv_err)?;
        Ok(values)
    }

    fn delete(&self, key: impl AsRef<[u8]>) -> Result<()> {
        self.tx.delete_cf(&self.cf_handle, key).map_err(okv_err)?;
        Ok(())
    }

    fn contains(&self, key: impl AsRef<[u8]>) -> Result<bool> {
        match self.tx.get_cf(&self.cf_handle, key).map_err(okv_err)? {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }
}

impl<'a, DB> DBColumnRef<'a> for RocksDBTransaction<'a, DB> {
    type Ref = DBPinnableSlice<'a>;

    fn get_ref(&'a self, key: impl AsRef<[u8]>) -> Result<Option<Self::Ref>> {
        let x = self
            .tx
            .get_pinned_cf(&self.cf_handle, key)
            .map_err(okv_err)?;
        let Some(x) = x else {
            return Ok(None);
        };
        Ok(Some(x))
    }
}

impl<'a, DB> DBColumnIterator for RocksDBTransaction<'a, DB> {
    fn iter(&self) -> Result<impl Iterator<Item = Result<(Vec<u8>, Vec<u8>)>>> {
        let iter = self
            .tx
            .iterator_cf(&self.cf_handle, rocksdb::IteratorMode::Start)
            .map(|v| match v {
                Ok((k, v)) => Ok((k.to_vec(), v.to_vec())),
                Err(e) => Err(okv_err(e)),
            });

        Ok(iter)
    }
}

impl<'a, DB> DBColumnIteratorPrefix for RocksDBTransaction<'a, DB> {
    fn iter_prefix(
        &self,
        prefix: impl AsRef<[u8]>,
    ) -> Result<impl Iterator<Item = Result<(Vec<u8>, Vec<u8>)>>> {
        let iter = self
            .tx
            .iterator_cf(
                &self.cf_handle,
                rocksdb::IteratorMode::From(prefix.as_ref(), rocksdb::Direction::Forward),
            )
            .map(|v| match v {
                Ok((k, v)) => Ok((k.to_vec(), v.to_vec())),
                Err(e) => Err(okv_err(e)),
            });

        Ok(iter)
    }
}
