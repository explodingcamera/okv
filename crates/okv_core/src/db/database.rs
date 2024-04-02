use crate::db::transactions::DatabaseTransaction;
use crate::env::Env;
use crate::error::Result;
use crate::{backend::*, traits::*, types::RefValue};

use inherent::inherent;
use std::marker::PhantomData;
use std::sync::Arc;

/// A collection of key-value pairs
#[derive(Clone)]
pub struct Database<K, V, D: DatabaseBackend> {
    name: String,
    pub(super) column: Arc<D::Column>,
    marker: PhantomData<(K, V)>,
}

impl<K, V, D: DatabaseBackend> Database<K, V, D> {
    pub(crate) fn new(env: Env<D>, name: &str) -> Result<Self> {
        let column = D::create_or_open(env, name)?;
        Ok(Self {
            name: name.to_string(),
            column: Arc::new(column),
            marker: PhantomData,
        })
    }

    /// Returns the name of the database.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns a new atomic reference to the database with different types.
    pub fn as_type<K2, V2>(self) -> Database<K2, V2, D> {
        Database {
            name: self.name,
            column: self.column,
            marker: PhantomData,
        }
    }
}

// All databases
#[inherent]
impl<Key, Val, D: DatabaseBackend> crate::traits::DBCommon<Key, Val> for Database<Key, Val, D> {
    /// Get the value from the database by `key`.
    pub fn get_raw(&self, key: impl AsRef<[u8]>) -> Result<Option<Vec<u8>>> {
        self.column.get(key)
    }

    /// Set a `key` to a value in the database.
    pub fn set_raw<'v>(&'v self, key: impl AsRef<[u8]>, val: &'v [u8]) -> Result<()> {
        self.column.set(key, val)
    }

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

    /// Set a `key` to a value in the database if the key does not exist.
    pub fn set_nx_raw<'v>(&'v self, key: impl AsRef<[u8]>, val: &'v [u8]) -> Result<bool> {
        self.column.set_nx(key, val)
    }

    /// Set a `key` to a serialized value in the database if the key does not exist.
    pub fn set_nx<'k, 'v>(&'v self, key: &'k <Key>::EItem, val: &'v <Val>::EItem) -> Result<bool>
    where
        Key: BytesEncode<'k>,
        Val: BytesEncode<'v>;

    /// Delete the serialized `val` from the database by `key`.
    pub fn delete<'k>(&self, key: &'k <Key>::EItem) -> Result<()>
    where
        Key: BytesEncode<'k>,
    {
        self.column.delete(Key::bytes_encode(key)?)
    }

    /// Get values from the database by `keys`.
    pub fn get_multi_raw<I, IV: AsRef<[u8]>>(&self, keys: I) -> Result<Vec<Option<Vec<u8>>>>
    where
        I: IntoIterator<Item = IV>,
    {
        keys.into_iter()
            .map(|key| self.column.get(key.as_ref()))
            .collect()
    }

    /// Get the serialized `val` from the database by `keys`.
    pub fn get_multi<'k, I>(&self, keys: I) -> Result<Vec<Option<<Val>::DItem>>>
    where
        Key: BytesEncode<'k>,
        I: IntoIterator<Item = &'k <Key>::EItem>,
        Val: BytesDecodeOwned;

    /// Check if the database contains the given key.
    pub fn contains<'k>(&self, key: &'k <Key>::EItem) -> Result<bool>
    where
        Key: BytesEncode<'k>,
    {
        self.column.contains(Key::bytes_encode(key)?)
    }
}

#[inherent]
impl<Key, Val, D, C: DBColumnClear> DBCommonClear for Database<Key, Val, D>
where
    D: DatabaseBackend<Column = C>,
{
    /// Clear the database, removing all key-value pairs.
    pub fn clear(&self) -> Result<()> {
        self.column.clear()
    }
}

#[inherent]
impl<Key, Val, D, C: DBColumnDelete> DBCommonDelete for Database<Key, Val, D>
where
    D: DatabaseBackend<Column = C>,
{
    /// Delete the database. Note that this will delete all data in the database.
    /// After calling this method, the database should not be used anymore or it
    /// will panic.
    pub fn delete_db(self) -> Result<()> {
        self.column.delete_db()
    }
}

#[inherent]
impl<'a, Key, Val, D, C> DBCommonRef<'a, Key, Val, C::Ref> for Database<Key, Val, D>
where
    C: DBColumnRef<'a> + 'a,
    D: DatabaseBackend<Column = C>,
{
    /// Get the serialized `val` from the database by `key`.
    ///
    /// See [`get_ref`](crate::traits::DBCommonRef::get_ref) for more information.
    pub fn get_ref<'k>(
        &'a self,
        key: &'k Key::EItem,
    ) -> Result<Option<RefValue<C::Ref, Val::DItem>>>
    where
        Key: BytesEncode<'k>,
        Val: BytesDecode<'a>,
    {
        let key_bytes = Key::bytes_encode(key)?;
        let val_bytes = self.column.get_ref(key_bytes.clone())?;

        match val_bytes {
            Some(val_bytes) => Ok(Some(RefValue {
                data: val_bytes,
                marker: PhantomData,
            })),
            None => Ok(None),
        }
    }
}

#[inherent]
impl<'a, Key, Val, D, C> DBCommonRefBatch<'a, Key, Val, C::Ref> for Database<Key, Val, D>
where
    C: DBColumnRefBatch<'a> + 'a,
    D: DatabaseBackend<Column = C>,
{
    /// Get the serialized `val` from the database by `key`.
    ///
    /// See [`get_multi_ref`](crate::traits::DBCommonRefBatch::get_multi_ref) for more information.
    #[allow(clippy::type_complexity)] // this isn't that complex
    pub fn get_multi_ref<'k, I>(
        &'a self,
        keys: I,
    ) -> Result<Vec<Option<RefValue<C::Ref, Val::DItem>>>>
    where
        Key: BytesEncode<'k>,
        Val: BytesDecode<'a>,
        I: IntoIterator<Item = &'k <Key>::EItem>,
    {
        let decoded_keys: Result<Vec<_>, _> =
            keys.into_iter().map(|key| Key::bytes_encode(key)).collect();
        let res = self.column.get_multi_ref(&decoded_keys?)?;
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

// Databases that support transactions
impl<'a, Key, Val, D, C> Database<Key, Val, D>
where
    C: DBColumnTransaction<'a> + 'a,
    D: DatabaseBackend<Column = C>,
{
    /// Clear the database, removing all key-value pairs.
    pub fn transaction(&'a self) -> Result<DatabaseTransaction<'a, Key, Val, C>> {
        Ok(DatabaseTransaction {
            column: self.column.transaction()?,
            marker: PhantomData,
        })
    }
}

// Databases that access to the underlying driver
impl<'a, K, V, D, C: DBColumn + 'a + Innerable> Database<K, V, D>
where
    D: DatabaseBackend<Column = C> + Innerable,
{
    /// Returns a reference to the underlying column.
    /// Can be used to access the database directly.
    pub fn inner(&self) -> &C {
        &self.column
    }
}

// Databases that support flushing
impl<K, V, D, C: DBColumn + Flushable> Database<K, V, D>
where
    D: DatabaseBackend<Column = C>,
{
    /// Returns a reference to the underlying column.
    /// Can be used to access the database directly.
    pub fn flush(&self) -> Result<()> {
        self.column.flush()
    }
}

// Databases that support iterating
#[inherent]
impl<K: BytesDecodeOwned, V: BytesDecodeOwned, D, C> DBCommonIter<K, V> for Database<K, V, D>
where
    for<'b> C: DBColumnIterator + 'b,
    D: DatabaseBackend<Column = C>,
{
    /// Get a iterator over the database, transforming raw bytes to `Key` and `Val` types.
    pub fn iter(&self) -> Result<impl Iterator<Item = Result<(K::DItem, V::DItem)>> + '_>;

    /// Iterate over all key-value pairs in the database.
    #[allow(refining_impl_trait)] // we need to specify the lifetime
    pub fn iter_raw(&self) -> Result<impl Iterator<Item = Result<(Vec<u8>, Vec<u8>)>> + '_> {
        self.column.iter()
    }
}

// Databases that support iterating with a prefix
impl<'a, K: BytesDecodeOwned, V: BytesDecodeOwned, D, C> DBCommonIterPrefix<'a, K, V>
    for Database<K, V, D>
where
    C: DBColumnIteratorPrefix + 'a,
    D: DatabaseBackend<Column = C>,
{
    /// Iterate over all key-value pairs in the database.
    fn iter_prefix_raw(
        &'a self,
        prefix: impl AsRef<[u8]>,
    ) -> Result<impl Iterator<Item = Result<(Vec<u8>, Vec<u8>)>>> {
        self.column.iter_prefix(prefix)
    }
}
