use std::cell::RefCell;

use okv_core::{
    backend::{DBColumn, DBColumnTransaction, DBTransaction},
    error::{Error, Result},
};
use ouroboros::self_referencing;
use redb::{ReadableTable, Table, TableDefinition};

use crate::{okv_err, RedbColumn};

type TxTable<'db, 'tx> = Table<'db, 'tx, &'static [u8], &'static [u8]>;
pub struct RedbTransaction<'a>(RefCell<RedbTxInner<'a>>);

// sadly this doesn't work with self_cell because the 'db lifetime is needed
// TODO: figure out how to make this work without interior mutability (RefCell)
// and self referencing
#[self_referencing]
struct RedbTxInner<'db> {
    tx: redb::WriteTransaction<'db>,
    table_def: TableDefinition<'db, &'static [u8], &'static [u8]>,

    #[borrows(tx)]
    #[covariant]
    table: TxTable<'db, 'this>,
}

impl RedbTransaction<'_> {
    // not a super nice solution, works for now
    // a seperate fn to create transactions with different durabilities would be better
    pub fn with_durability(self, durability: redb::Durability) -> Result<Self> {
        let mut inner = self.0.into_inner().into_heads();
        inner.tx.set_durability(durability);

        let tx = RedbTxInnerTryBuilder {
            table_builder: |tx| {
                let table = tx.open_table(inner.table_def).map_err(okv_err)?;
                Result::<_, Error>::Ok(table)
            },
            table_def: inner.table_def,
            tx: inner.tx,
        }
        .try_build()?;
        Ok(RedbTransaction(tx.into()))
    }
}

impl<'a> DBColumn for RedbTransaction<'a> {
    fn contains(&self, key: impl AsRef<[u8]>) -> Result<bool> {
        let inner = self.0.borrow();
        let res = inner.borrow_table().get(key.as_ref()).map_err(okv_err)?;
        Ok(res.is_some())
    }
    fn delete(&self, key: impl AsRef<[u8]>) -> Result<()> {
        self.0.borrow_mut().with_table_mut(|table| {
            table.remove(key.as_ref()).map_err(okv_err)?;
            Ok(())
        })
    }
    fn get(&self, key: impl AsRef<[u8]>) -> Result<Option<Vec<u8>>> {
        let inner = self.0.borrow();
        let res = inner.borrow_table().get(key.as_ref()).map_err(okv_err)?;
        Ok(res.map(|v| v.value().to_vec()))
    }
    fn get_multi<I>(&self, keys: I) -> Result<Vec<Option<Vec<u8>>>>
    where
        I: IntoIterator,
        I::Item: AsRef<[u8]>,
    {
        let mut res = Vec::new();
        let inner = self.0.borrow();
        let table = inner.borrow_table();
        for key in keys {
            let val = table.get(key.as_ref()).map_err(okv_err)?;
            res.push(val.map(|v| v.value().to_vec()));
        }
        Ok(res)
    }
    fn set(&self, key: impl AsRef<[u8]>, val: impl AsRef<[u8]>) -> Result<()> {
        self.0.borrow_mut().with_table_mut(|table| {
            table.insert(key.as_ref(), val.as_ref()).map_err(okv_err)?;
            Ok(())
        })
    }
}

impl<'a> DBTransaction for RedbTransaction<'a> {
    fn commit(self) -> Result<()> {
        let inner = self.0.into_inner();
        let heads = inner.into_heads();
        heads.tx.commit().map_err(okv_err)?;
        Ok(())
    }

    fn rollback(self) -> Result<()> {
        let inner = self.0.into_inner();
        let heads = inner.into_heads();
        heads.tx.abort().map_err(okv_err)?;
        Ok(())
    }
}

impl<'a> DBColumnTransaction<'a> for RedbColumn {
    type Txn = RedbTransaction<'a>;

    fn transaction(&'a self) -> Result<Self::Txn> {
        let tx = RedbTxInnerTryBuilder {
            table_builder: |tx| {
                let table = tx.open_table(self.table()).map_err(okv_err)?;
                Result::<_, Error>::Ok(table)
            },
            table_def: self.table(),
            tx: self.db().begin_write().map_err(okv_err)?,
        }
        .try_build()?;

        Ok(RedbTransaction(tx.into()))
    }
}
