use std::marker::PhantomData;

use inherent::inherent;

use crate::{
    backend::{DBColumn, DBColumnTransaction, DBTransaction, DatabaseBackend},
    BytesDecodeOwned, BytesEncode, Env, Result,
};

pub struct DatabaseTransaction<'a, K, V, D, C>
where
    C: DBColumnTransaction<'a>,
    D: DatabaseBackend<'a, Column = C>,
{
    pub(super) _env: &'a Env<'a, D>,
    pub(super) column: C::Txn,
    pub(super) _phantom: PhantomData<(K, V)>,
}

impl<'a, K, V, D, C> DatabaseTransaction<'a, K, V, D, C>
where
    C: DBColumnTransaction<'a>,
    D: DatabaseBackend<'a, Column = C>,
{
    /// Commit the transaction.
    pub fn commit(self) -> Result<()> {
        self.column.commit()
    }

    /// Rollback the transaction.
    pub fn rollback(self) -> Result<()> {
        self.column.rollback()
    }
}

#[inherent]
impl<'a, Key, Val, D, C> crate::traits::DBCommon<Key, Val>
    for DatabaseTransaction<'a, Key, Val, D, C>
where
    C: DBColumnTransaction<'a>,
    D: DatabaseBackend<'a, Column = C>,
{
    /// Get the value from the database by `key`.
    pub fn get_raw(&self, key: impl AsRef<[u8]>) -> Result<Option<Vec<u8>>> {
        let res = self.column.get(key)?;
        Ok(res)
    }

    /// Set a `key` to a value in the database.
    pub fn set_raw<'v>(&'v mut self, key: impl AsRef<[u8]>, val: &'v [u8]) -> Result<()> {
        self.column.set(key, val)?;
        Ok(())
    }

    /// Get the serialized `val` from the database by `key`.
    pub fn get<'k, 'v>(&self, key: &'k <Key>::EItem) -> Result<<Val>::DItem>
    where
        Key: BytesEncode<'k>,
        Val: BytesDecodeOwned;

    /// Set a `key` to the serialized `val` in the database.
    pub fn set<'k, 'v>(&'v mut self, key: &'k <Key>::EItem, val: &'v <Val>::EItem) -> Result<()>
    where
        Key: BytesEncode<'k>,
        Val: BytesEncode<'v>;

    /// Delete the serialized `val` from the database by `key`.
    pub fn delete<'k>(&mut self, key: &'k <Key>::EItem) -> Result<()>
    where
        Key: BytesEncode<'k>,
    {
        let key_bytes = Key::bytes_encode(key)?;
        self.column.delete(key_bytes)?;
        Ok(())
    }

    /// Get the serialized `val` from the database by `key`.
    pub fn get_multi<'k, I>(&self, keys: I) -> Result<Vec<Option<Val::DItem>>>
    where
        Key: BytesEncode<'k>,
        I: IntoIterator<Item = &'k Key::EItem>,
        Val: BytesDecodeOwned;

    /// Get the `val` from the database by `key`.
    pub fn get_multi_raw<I, IV: AsRef<[u8]>>(&self, keys: I) -> Result<Vec<Option<Vec<u8>>>>
    where
        I: IntoIterator<Item = IV>,
    {
        let val_bytes = self.column.get_multi(keys)?;
        Ok(val_bytes)
    }

    /// Check if the database contains the given key.
    pub fn contains<'k>(&self, key: &'k <Key>::EItem) -> Result<bool>
    where
        Key: BytesEncode<'k>,
    {
        let key_bytes = Key::bytes_encode(key)?;
        let res = self.column.contains(key_bytes)?;
        Ok(res)
    }
}
