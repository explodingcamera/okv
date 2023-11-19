use std::marker::PhantomData;
use std::sync::Arc;

use crate::backend::{DatabaseBackend, DatabaseColumn, DatabaseColumnRef};
use crate::env::Env;
use crate::traits::{BytesDecode, BytesDecodeOwned, BytesEncode};
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

impl<'b, 'c, Key, Val, D, C> Database<'b, 'c, Key, Val, D>
where
    C: DatabaseColumn<'c> + 'b,
    D: DatabaseBackend<'b, 'c, Column = C>,
{
    /// Set a `key` in the database to the serialized value of `val`.
    pub fn set<'k, 'v>(&'v mut self, key: &'k Key::EItem, val: &'v Val::EItem) -> Result<()>
    where
        Key: BytesEncode<'k>,
        Val: BytesEncode<'v>,
    {
        let key_bytes = Key::bytes_encode(key)?;
        let val_bytes = Val::bytes_encode(val)?;
        self.0.column.set(key_bytes, &val_bytes)?;
        Ok(())
    }

    /// Get a `key` from the database and deserialize it.
    pub fn get<'k, 'v>(&self, key: &'k Key::EItem) -> Result<Val::DItem>
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
}

impl<'b, 'c, Key, Val, D, C> Database<'b, 'c, Key, Val, D>
where
    C: DatabaseColumnRef<'c> + 'b,
    D: DatabaseBackend<'b, 'c, Column = C>,
{
    /// Get a `key` from the database. To deserialize the value, use [`crate::db::RefValue::deserialize()`].
    /// Allows for more efficient access to the underlying bytes by returning a reference.
    pub fn get_ref<'v, 'k>(&'v self, key: &'k Key::EItem) -> Result<RefValue<C::Ref, Val::DItem>>
    where
        Key: BytesEncode<'k>,
        Val: BytesDecode<'v>,
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

/// A reference to a value in the database.
/// Allows for more efficient access to the underlying bytes by returning a reference.
/// To deserialize the value, use [`crate::db::RefValue::deserialize()`].
pub struct RefValue<'a, T, Val> {
    data: T,
    marker: PhantomData<&'a Val>,
}

impl<'a, T, Val> RefValue<'a, T, Val>
where
    T: AsRef<[u8]> + 'a + std::ops::Deref<Target = [u8]> + Send + Sync,
    Val: BytesDecode<'a>,
{
    /// Returns a reference to the inner value.
    pub fn inner(&self) -> &T {
        &self.data
    }

    /// Deserialize the value from the database to the type `Val`.
    pub fn deserialize(&'a self) -> Result<Val::DItem> {
        Ok(Val::bytes_decode(&self.data)?)
    }
}
