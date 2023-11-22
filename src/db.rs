use std::marker::PhantomData;
use std::sync::Arc;

use inherent::inherent;

use crate::backend::{
    DatabaseBackend, DatabaseCommon, DatabaseCommonClear, DatabaseCommonRef, DatabaseCommonRefBatch,
};
use crate::env::Env;
use crate::traits::{BytesDecode, BytesDecodeOwned, BytesEncode};
use crate::types::RefValue;
use crate::{Flushable, Innerable, Result};

/// A collection of key-value pairs
/// Can be cloned.
pub struct Database<'a, K, V, D>(Arc<DatabaseInner<'a, K, V, D>>)
where
    D: DatabaseBackend<'a>;

impl<'a, K, V, D> Clone for Database<'a, K, V, D>
where
    D: DatabaseBackend<'a>,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<'a, K, V, D, C> Database<'a, K, V, D>
where
    C: DatabaseCommon + 'a + Innerable,
    D: DatabaseBackend<'a, Column = C> + Innerable,
{
    /// Returns a reference to the underlying column.
    /// Can be used to access the database directly.
    pub fn inner(&self) -> &C {
        &self.0.column
    }
}

impl<'a, K, V, D, C> Database<'a, K, V, D>
where
    C: DatabaseCommon + Flushable,
    D: DatabaseBackend<'a, Column = C>,
{
    /// Returns a reference to the underlying column.
    /// Can be used to access the database directly.
    pub fn flush(&self) -> Result<()> {
        self.0.column.flush()
    }
}

struct DatabaseInner<'a, K, V, D>
where
    D: DatabaseBackend<'a>,
{
    _name: String,
    _env: &'a Env<'a, D>,
    column: D::Column,
    _phantom: PhantomData<(K, V)>,
}

pub(crate) fn new<'a, K, V, D>(env: &'a Env<'a, D>, name: &str) -> Result<Database<'a, K, V, D>>
where
    D: DatabaseBackend<'a>,
{
    let column = env.db().create_or_open(name)?;
    Ok(Database(Arc::new(DatabaseInner {
        _name: name.to_string(),
        _env: env,
        column,
        _phantom: PhantomData,
    })))
}

#[inherent]
impl<'a, Key, Val, D> crate::traits::DBCommon<Key, Val> for Database<'a, Key, Val, D>
where
    D: DatabaseBackend<'a>,
{
    /// Get the serialized `val` from the database by `key`.
    pub fn get<'k, 'v>(&self, key: &'k <Key>::EItem) -> Result<<Val>::DItem>
    where
        Key: BytesEncode<'k>,
        Val: BytesDecodeOwned,
    {
        let key_bytes = Key::bytes_encode(key)?;

        let val_bytes =
            self.0
                .column
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
        self.0.column.set(key_bytes, &val_bytes)?;
        Ok(())
    }

    /// Delete the serialized `val` from the database by `key`.
    pub fn delete<'k>(&mut self, key: &'k <Key>::EItem) -> Result<()>
    where
        Key: BytesEncode<'k>,
    {
        let key_bytes = Key::bytes_encode(key)?;
        self.0.column.delete(key_bytes)?;
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
            let val_bytes = self.0.column.get(key_bytes)?;
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
        let res = self.0.column.contains(key_bytes)?;
        Ok(res)
    }
}

#[inherent]
impl<'a, Key, Val, D, C> crate::DBCommonClear for Database<'a, Key, Val, D>
where
    C: DatabaseCommonClear,
    D: DatabaseBackend<'a, Column = C>,
{
    /// Clear the database, removing all key-value pairs.
    pub fn clear(&self) -> Result<()> {
        self.0.column.clear()?;
        Ok(())
    }
}

#[inherent]
impl<'a, Key, Val, D, C> crate::DBCommonRef<'a, Key, Val, C::Ref> for Database<'a, Key, Val, D>
where
    C: DatabaseCommonRef<'a> + 'a,
    D: DatabaseBackend<'a, Column = C>,
{
    /// Get the serialized `val` from the database by `key`.
    ///
    /// See [`get_ref`](crate::DBCommonRef::get_ref) for more information.
    pub fn get_ref<'k>(&'a self, key: &'k Key::EItem) -> Result<RefValue<C::Ref, Val::DItem>>
    where
        Key: BytesEncode<'k>,
        Val: BytesDecode<'a>,
    {
        let key_bytes = Key::bytes_encode(key)?;

        let val_bytes =
            self.0
                .column
                .get_ref(key_bytes.clone())?
                .ok_or_else(|| crate::Error::KeyNotFound {
                    key: key_bytes.to_vec(),
                })?;

        Ok(RefValue {
            data: val_bytes,
            marker: PhantomData,
        })
    }
}

#[inherent]
impl<'a, Key, Val, D, C> crate::DBCommonRefBatch<'a, Key, Val, C::Ref> for Database<'a, Key, Val, D>
where
    C: DatabaseCommonRefBatch<'a>,
    D: DatabaseBackend<'a, Column = C>,
{
    /// Get the serialized `val` from the database by `key`.
    ///
    /// See [`get_multi_ref`](crate::DBCommonRef::get_multi_ref) for more information.
    #[allow(clippy::type_complexity)] // trait associated types are not stable yet
    pub fn get_multi_ref<'k, I>(&self, keys: I) -> Result<Vec<Option<RefValue<C::Ref, Val::DItem>>>>
    where
        Key: BytesEncode<'k>,
        I: IntoIterator<Item = &'k <Key>::EItem>,
        Val: BytesDecode<'a>,
    {
        let decoded_keys: Result<Vec<_>, _> =
            keys.into_iter().map(|key| Key::bytes_encode(key)).collect();
        let res = self.0.column.get_multi_ref(decoded_keys?)?;
        let wrapped_res = res
            .into_iter()
            .map(|val| {
                val.map(|val| RefValue {
                    data: val,
                    marker: PhantomData,
                })
            })
            .collect();
        Ok(wrapped_res)
    }
}
