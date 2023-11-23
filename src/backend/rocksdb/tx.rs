use std::sync::Arc;

use rocksdb::{DBPinnableSlice, OptimisticTransactionDB, TransactionDB};

use crate::{
    backend::{DBColumn, DBColumnRef, DBColumnTransaction, DBTransaction},
    Result,
};

use super::{RocksDbOptimisticColumn, RocksDbPessimisticColumn};

pub struct RocksDBTransaction<'a, DB> {
    pub(crate) cf_handle: Arc<rocksdb::BoundColumnFamily<'a>>,
    tx: rocksdb::Transaction<'a, DB>,
}

impl<'a> DBColumnTransaction<'a> for RocksDbOptimisticColumn {
    type Txn = RocksDBTransaction<'a, OptimisticTransactionDB>;

    fn transaction(&'a self) -> crate::Result<Self::Txn> {
        let tx = self.db().transaction();
        Ok(RocksDBTransaction {
            tx,
            cf_handle: self.cf_handle().clone(),
        })
    }
}

impl<'a> DBColumnTransaction<'a> for RocksDbPessimisticColumn {
    type Txn = RocksDBTransaction<'a, TransactionDB>;

    fn transaction(&'a self) -> crate::Result<Self::Txn> {
        let tx = self.db().transaction();
        Ok(RocksDBTransaction {
            tx,
            cf_handle: self.cf_handle().clone(),
        })
    }
}

impl<'a, DB> DBTransaction for RocksDBTransaction<'a, DB> {
    fn commit(self) -> crate::Result<()> {
        self.tx.commit()?;
        Ok(())
    }

    fn rollback(self) -> crate::Result<()> {
        self.tx.rollback()?;
        Ok(())
    }
}

impl<'a, DB> DBColumn for RocksDBTransaction<'a, DB> {
    fn set(&self, key: impl AsRef<[u8]>, val: impl AsRef<[u8]>) -> crate::Result<()> {
        self.tx.put_cf(&self.cf_handle, key, val)?;
        Ok(())
    }

    fn get(&self, key: impl AsRef<[u8]>) -> crate::Result<Option<Vec<u8>>> {
        match self.tx.get_cf(&self.cf_handle, key)? {
            Some(x) => Ok(Some(x.to_vec())),
            None => Ok(None),
        }
    }

    fn get_multi<I>(&self, keys: I) -> crate::Result<Vec<Option<Vec<u8>>>>
    where
        I: IntoIterator,
        I::Item: AsRef<[u8]>,
    {
        let keys = keys.into_iter().map(|key| (&self.cf_handle, key));
        let values = self.tx.multi_get_cf(keys);
        let values = values
            .into_iter()
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(values)
    }

    fn delete(&self, key: impl AsRef<[u8]>) -> crate::Result<()> {
        self.tx.delete_cf(&self.cf_handle, key)?;
        Ok(())
    }

    fn contains(&self, key: impl AsRef<[u8]>) -> crate::Result<bool> {
        match self.tx.get_cf(&self.cf_handle, key)? {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }
}

impl<'a, DB> DBColumnRef<'a> for RocksDBTransaction<'a, DB> {
    type Ref = DBPinnableSlice<'a>;

    fn get_ref(&'a self, key: impl AsRef<[u8]>) -> Result<Option<Self::Ref>> {
        let x = self.tx.get_pinned_cf(&self.cf_handle, key)?;
        let Some(x) = x else {
            return Ok(None);
        };
        Ok(Some(x))
    }
}
