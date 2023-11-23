use std::marker::PhantomData;

use inherent::inherent;

use crate::{
    backend::{DBColumn, DBColumnTransaction, DBTransaction},
    BytesDecodeOwned, BytesEncode, Result,
};

pub struct DatabaseTransaction<'a, K, V, C>
where
    C: DBColumnTransaction<'a>,
{
    pub(super) column: C::Txn,
    pub(super) _phantom: PhantomData<(K, V)>,
}

impl<'a, K, V, C> DatabaseTransaction<'a, K, V, C>
where
    C: DBColumnTransaction<'a>,
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
impl<'a, Key, Val, C> crate::traits::DBCommon<Key, Val> for DatabaseTransaction<'a, Key, Val, C>
where
    C: DBColumnTransaction<'a>,
{
    /// Get the value from the database by `key`.
    pub fn get_raw(&self, key: impl AsRef<[u8]>) -> Result<Option<Vec<u8>>> {
        let res = self.column.get(key)?;
        Ok(res)
    }

    /// Set a `key` to a value in the database.
    pub fn set_raw<'v>(&'v self, key: impl AsRef<[u8]>, val: &'v [u8]) -> Result<()> {
        self.column.set(key, val)?;
        Ok(())
    }

    /// Set a `key` to a value in the database if the key does not exist.
    pub fn set_nx_raw<'v>(&'v self, key: impl AsRef<[u8]>, val: &'v [u8]) -> Result<bool> {
        let res = self.column.set_nx(key, val)?;
        Ok(res)
    }

    /// Set a `key` to a serialized value in the database if the key does not exist.
    pub fn set_nx<'k, 'v>(&'v self, key: &'k <Key>::EItem, val: &'v <Val>::EItem) -> Result<bool>
    where
        Key: BytesEncode<'k>,
        Val: BytesEncode<'v>;

    /// Get the serialized `val` from the database by `key`.
    pub fn get<'k, 'v>(&self, key: &'k <Key>::EItem) -> Result<Option<<Val>::DItem>>
    where
        Key: BytesEncode<'k>,
        Val: BytesDecodeOwned;

    /// Set a `key` to the serialized `val` in the database.
    pub fn set<'k, 'v>(&'v self, key: &'k <Key>::EItem, val: &'v <Val>::EItem) -> Result<()>
    where
        Key: BytesEncode<'k>,
        Val: BytesEncode<'v>;

    /// Delete the serialized `val` from the database by `key`.
    pub fn delete<'k>(&self, key: &'k <Key>::EItem) -> Result<()>
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
