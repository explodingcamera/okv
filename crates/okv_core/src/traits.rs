use std::borrow::Cow;

use crate::error::{DecodeError, EncodeError, Result};
use crate::types::RefValue;

/// A trait that represents a flushable structure.
/// This is used to flush the database on supported backends.
pub trait Flushable {
    /// Flush the database to disk.
    fn flush(&self) -> Result<()>;
}

/// A trait that represents an innerable structure.
/// This is used to access the database directly.
pub trait Innerable {
    /// The inner type
    type Inner;

    /// Get a reference to the inner type
    fn inner(&self) -> &Self::Inner;
}

/// A trait that represents an encoding structure.
pub trait BytesEncode<'a> {
    /// The type to encode
    type EItem: ?Sized + 'a;

    /// Encode the given item as bytes
    fn bytes_encode(item: &'a Self::EItem) -> Result<Cow<'a, [u8]>, EncodeError>;
}

/// A trait that represents a decoding structure.
pub trait BytesDecode<'a> {
    /// The type to decode
    type DItem: 'a;

    /// Decode the given bytes as DItem
    fn bytes_decode(bytes: &'a [u8]) -> Result<Self::DItem, DecodeError>;
}

/// A trait that represents a decoding structure that owns the data.
pub trait BytesDecodeOwned {
    /// The type to decode to
    type DItem;

    /// Decode the given bytes as DItem
    fn bytes_decode_owned(bytes: &[u8]) -> Result<Self::DItem, DecodeError>;
}

/// A trait that represents a common database interface.
pub trait DBCommon<Key, Val> {
    /// Set a key to a value in the database.
    fn set_raw<'v>(&'v self, key: impl AsRef<[u8]>, val: &'v [u8]) -> Result<()>;

    /// Set a `key` to a value in the database if the key does not exist.
    fn set_nx_raw<'v>(&'v self, key: impl AsRef<[u8]>, val: &'v [u8]) -> Result<bool>;

    /// Delete the serialized `val` from the database by `key`.
    fn delete<'k>(&self, key: &'k Key::EItem) -> Result<()>
    where
        Key: BytesEncode<'k>;

    /// Check if the database contains the given key.
    fn contains<'k>(&self, key: &'k Key::EItem) -> Result<bool>
    where
        Key: BytesEncode<'k>;

    /// Set a `key` to the serialized `val` in the database.
    fn set<'k, 'v>(&'v self, key: &'k Key::EItem, val: &'v Val::EItem) -> Result<()>
    where
        Key: BytesEncode<'k>,
        Val: BytesEncode<'v>,
    {
        self.set_raw(Key::bytes_encode(key)?, &Val::bytes_encode(val)?)
    }

    /// Set a `key` to a serialized value in the database if the key does not exist.
    fn set_nx<'k, 'v>(&'v self, key: &'k Key::EItem, val: &'v Val::EItem) -> Result<bool>
    where
        Key: BytesEncode<'k>,
        Val: BytesEncode<'v>,
    {
        self.set_nx_raw(Key::bytes_encode(key)?, &Val::bytes_encode(val)?)
    }

    /// Set a `key` to a value in the database.
    fn get_raw(&self, key: impl AsRef<[u8]>) -> Result<Option<Vec<u8>>>;

    /// Get the serialized `val` from the database by `key`.
    fn get<'k, 'v>(&self, key: &'k Key::EItem) -> Result<Option<Val::DItem>>
    where
        Key: BytesEncode<'k>,
        Val: BytesDecodeOwned,
    {
        let key_bytes = Key::bytes_encode(key)?;
        let val_bytes = self.get_raw(key_bytes)?;
        match val_bytes {
            Some(val_bytes) => Ok(Some(Val::bytes_decode_owned(&val_bytes)?)),
            None => Ok(None),
        }
    }

    /// Get values from the database by `keys`.
    fn get_multi_raw<I, IV: AsRef<[u8]>>(&self, keys: I) -> Result<Vec<Option<Vec<u8>>>>
    where
        I: IntoIterator<Item = IV>;

    /// Get the serialized `val` from the database by `key`.
    fn get_multi<'k, I>(&self, keys: I) -> Result<Vec<Option<Val::DItem>>>
    where
        Key: BytesEncode<'k>,
        I: IntoIterator<Item = &'k Key::EItem>,
        Val: BytesDecodeOwned,
    {
        let encoded_keys: Result<Vec<Vec<u8>>, EncodeError> = keys
            .into_iter()
            .map(|key| Key::bytes_encode(key).map(|cow| cow.into_owned()))
            .collect();

        let res = self
            .get_multi_raw(encoded_keys?)?
            .iter()
            .map(|item| match item {
                Some(val_bytes) => Ok(Some(Val::bytes_decode_owned(val_bytes)?)),
                None => Ok(None),
            })
            .collect::<Result<Vec<Option<Val::DItem>>>>()?;

        Ok(res)
    }
}

/// A trait that represents a common database interface can be cleared.
pub trait DBCommonClear {
    /// Clear the database, removing all key-value pairs.
    fn clear(&self) -> Result<()>;
}

/// A database that supports deletion.
pub trait DBCommonDelete {
    /// Clear the database, removing all key-value pairs.
    fn delete_db(self) -> Result<()>;
}

/// A database that can return references.
pub trait DBCommonRef<'c, Key, Val, Ref>
where
    Ref: AsRef<[u8]> + 'c + std::ops::Deref<Target = [u8]> + Send + Sync,
{
    /// Get the serialized `val` from the database by `key`.
    /// Prefer this method over `get` for efficiency when only a reference to the value is needed
    /// and your backend supports it.
    fn get_ref<'k>(&'c self, key: &'k Key::EItem) -> Result<Option<RefValue<Ref, Val::DItem>>>
    where
        Key: BytesEncode<'k>,
        Val: BytesDecode<'c>;
}

// FUTURE: use impl Trait when it's stable (https://github.com/rust-lang/rust/pull/115822)
/// A database that can return references in batches.
pub trait DBCommonRefBatch<'c, Key, Val, Ref>
where
    Ref: AsRef<[u8]> + 'c + std::ops::Deref<Target = [u8]> + Send + Sync,
{
    /// Get the serialized `val` from the database by `key`.
    /// Use this method over `get_multi` for efficiency when you only need a reference to the value
    /// and your backend supports it.
    #[allow(clippy::type_complexity)] // not that complex really
    fn get_multi_ref<'k, I>(&'c self, keys: I) -> Result<Vec<Option<RefValue<Ref, Val::DItem>>>>
    where
        Key: BytesEncode<'k>,
        I: IntoIterator<Item = &'k Key::EItem>,
        Val: BytesDecode<'c>;
}

// FUTURE: use impl Trait when it's stable (https://github.com/rust-lang/rust/pull/115822)
/// A database that supports iterators.
pub trait DBCommonIter<Key, Val> {
    /// Get a raw iterator over the database.
    fn iter_raw(&self) -> Result<DBIterator<Vec<u8>, Vec<u8>>>;

    /// Get a iterator over the database, transforming raw bytes to `Key` and `Val` types.
    #[allow(clippy::type_complexity)] // not that complex really
    fn iter(&self) -> Result<DBIterator<Key::DItem, Val::DItem>>
    where
        Val: BytesDecodeOwned,
        Key: BytesDecodeOwned,
    {
        let raw_iterator = self.iter_raw()?;
        let decoded_iterator = raw_iterator.map(|item| {
            let (key_bytes, val_bytes) = item?;
            let key = Key::bytes_decode_owned(&key_bytes)?;
            let val = Val::bytes_decode_owned(&val_bytes)?;
            Ok((key, val))
        });
        Ok(Box::new(decoded_iterator))
    }
}

/// A database that supports iterators over a prefix.
pub trait DBCommonIterPrefix<'c, Key, Val> {
    /// Get a raw iterator over the database for a given byte prefix.
    fn iter_prefix_raw(&'c self, prefix: impl AsRef<[u8]>) -> Result<DBIterator<Vec<u8>, Vec<u8>>>;

    /// Get a iterator over the database, transforming raw bytes to `Key` and `Val` types.
    fn iter_prefix<'k, Prefix>(
        &'c self,
        prefix: &'k Prefix::EItem,
    ) -> Result<DBIterator<Key::DItem, Val::DItem>>
    where
        Val: BytesDecodeOwned,
        Key: BytesDecodeOwned,
        Prefix: BytesEncode<'k>,
    {
        let prefix_bytes = Prefix::bytes_encode(prefix)?;
        let raw_iterator = self.iter_prefix_raw(prefix_bytes)?;

        let decoded_iterator = raw_iterator.map(|item| {
            let (key_bytes, val_bytes) = item?;
            let key = Key::bytes_decode_owned(&key_bytes)?;
            let val = Val::bytes_decode_owned(&val_bytes)?;
            Ok((key, val))
        });

        Ok(Box::new(decoded_iterator))
    }
}

/// An iterator over a database.
pub type DBIterator<'c, Key, Val> = Box<dyn Iterator<Item = Result<(Key, Val)>> + 'c>;
