use std::marker::PhantomData;
use std::sync::Arc;

use inherent::inherent;

use crate::backend::{DatabaseBackend, DatabaseColumn, DatabaseColumnRef, Flushable};
use crate::env::Env;
use crate::traits::{BytesDecode, BytesDecodeOwned, BytesEncode};
use crate::types::RefValue;
use crate::Result;

/// A collection of key-value pairs
/// Can be cloned.
pub struct Database<'b, 'c, K, V, D>(Arc<DatabaseInner<'b, 'c, K, V, D>>)
where
    D: DatabaseBackend<'b, 'c>;

impl<'b, 'c, K, V, D> Clone for Database<'b, 'c, K, V, D>
where
    D: DatabaseBackend<'b, 'c>,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<'b, 'c, K, V, D, C> Database<'b, 'c, K, V, D>
where
    C: DatabaseColumn + 'b,
    D: DatabaseBackend<'b, 'c, Column = C>,
{
    /// Returns a reference to the underlying column.
    /// Can be used to access the database directly.
    pub fn inner(&self) -> &C::Inner {
        self.0.column.inner()
    }
}

impl<'b, 'c, K, V, D, C> Database<'b, 'c, K, V, D>
where
    C: DatabaseColumn + 'b + Flushable,
    D: DatabaseBackend<'b, 'c, Column = C>,
{
    /// Returns a reference to the underlying column.
    /// Can be used to access the database directly.
    pub fn flush(&self) -> Result<()> {
        self.0.column.flush()
    }
}

struct DatabaseInner<'b, 'c, K, V, D>
where
    D: DatabaseBackend<'b, 'c>,
{
    _name: String,
    _env: &'b Env<'b, 'c, D>,
    column: D::Column,
    _phantom: PhantomData<(K, V)>,
}

pub(crate) fn new<'b, 'c, K, V, D>(
    env: &'b Env<'b, 'c, D>,
    name: &str,
) -> Result<Database<'b, 'c, K, V, D>>
where
    D: DatabaseBackend<'b, 'c>,
    'b: 'c,
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
impl<'b, 'c, Key, Val, D> crate::traits::DBCommon<Key, Val> for Database<'b, 'c, Key, Val, D>
where
    D: DatabaseBackend<'b, 'c>,
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

    /// Clear the database, removing all key-value pairs.
    pub fn clear(&mut self) -> Result<()> {
        self.0.column.clear()?;
        Ok(())
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
impl<'b, 'c, Key, Val, D, C> crate::DBCommonRef<'c, Key, Val, C::Ref>
    for Database<'b, 'c, Key, Val, D>
where
    C: DatabaseColumnRef<'c> + 'b,
    D: DatabaseBackend<'b, 'c, Column = C>,
{
    /// Get the serialized `val` from the database by `key`.
    ///
    /// See [`get_ref`](crate::DBCommonRef::get_ref) for more information.
    pub fn get_ref<'v, 'k>(&'v self, key: &'k Key::EItem) -> Result<RefValue<C::Ref, Val::DItem>>
    where
        Key: BytesEncode<'k>,
        Val: BytesDecode<'c>,
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

    /// Get the serialized `val` from the database by `key`.
    ///
    /// See [`get_multi_ref`](crate::DBCommonRef::get_multi_ref) for more information.
    #[allow(clippy::type_complexity)] // trait associated types are not stable yet
    pub fn get_multi_ref<'k, I>(&self, keys: I) -> Result<Vec<Option<RefValue<C::Ref, Val::DItem>>>>
    where
        Key: BytesEncode<'k>,
        I: IntoIterator<Item = &'k <Key>::EItem>,
        Val: BytesDecode<'c>,
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
