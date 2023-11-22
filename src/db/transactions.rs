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
    /// Get the serialized `val` from the database by `key`.
    pub fn get<'k, 'v>(&self, key: &'k <Key>::EItem) -> Result<<Val>::DItem>
    where
        Key: BytesEncode<'k>,
        Val: BytesDecodeOwned,
    {
        let key_bytes = Key::bytes_encode(key)?;

        let val_bytes =
            self.column
                .get(key_bytes.clone())?
                .ok_or_else(|| crate::Error::KeyNotFound {
                    key: key_bytes.to_vec(),
                })?;

        let res = Val::bytes_decode_owned(&val_bytes)?;
        Ok(res)
    }

    /// Set a `key` to the serialized `val` in the database.
    pub fn set<'k, 'v>(&'v mut self, key: &'k <Key>::EItem, val: &'v <Val>::EItem) -> Result<()>
    where
        Key: BytesEncode<'k>,
        Val: BytesEncode<'v>,
    {
        let key_bytes = Key::bytes_encode(key)?;
        let val_bytes = Val::bytes_encode(val)?;
        self.column.set(key_bytes, &val_bytes)?;
        Ok(())
    }

    /// Delete the serialized `val` from the database by `key`.
    pub fn delete<'k>(&mut self, key: &'k <Key>::EItem) -> Result<()>
    where
        Key: BytesEncode<'k>,
    {
        let key_bytes = Key::bytes_encode(key)?;
        self.column.delete(key_bytes)?;
        Ok(())
    }

    /// Get the serialized `val` from the database by `keys`.
    pub fn get_multi<'k, I>(&self, keys: I) -> Result<Vec<Option<<Val>::DItem>>>
    where
        Key: BytesEncode<'k>,
        I: IntoIterator<Item = &'k <Key>::EItem>,
        Val: BytesDecodeOwned,
    {
        let mut res = Vec::new();
        for key in keys {
            let key_bytes = Key::bytes_encode(key)?;
            let val_bytes = self.column.get(key_bytes)?;
            let val = match val_bytes {
                Some(val_bytes) => Some(Val::bytes_decode_owned(&val_bytes)?),
                None => None,
            };
            res.push(val);
        }
        Ok(res)
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
