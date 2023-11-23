use std::borrow::Cow;

use crate::{types::RefValue, DecodeError, EncodeError, Error, Result};

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
    fn set_nx_raw<'k, 'v>(&'v self, key: impl AsRef<[u8]>, val: &'v [u8]) -> Result<bool>;

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
        let key_bytes = Key::bytes_encode(key)?;
        let val_bytes = Val::bytes_encode(val)?;
        self.set_raw(key_bytes.as_ref(), val_bytes.as_ref())?;
        Ok(())
    }

    /// Set a `key` to a serialized value in the database if the key does not exist.
    fn set_nx<'k, 'v>(&'v self, key: &'k Key::EItem, val: &'v Val::EItem) -> Result<bool>
    where
        Key: BytesEncode<'k>,
        Val: BytesEncode<'v>,
    {
        let key_bytes = Key::bytes_encode(key)?;
        let val_bytes = Val::bytes_encode(val)?;
        self.set_nx_raw(key_bytes.as_ref(), val_bytes.as_ref())
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

        let val_bytes = self.get_raw(&key_bytes)?;
        match val_bytes {
            Some(val_bytes) => {
                let res = Val::bytes_decode_owned(&val_bytes)?;
                Ok(Some(res))
            }
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
        let mut encoded_keys: Vec<Vec<u8>> = vec![];
        for key in keys {
            let key_bytes = Key::bytes_encode(key)?;
            encoded_keys.push(key_bytes.to_vec());
        }

        let res = self.get_multi_raw(encoded_keys)?;
        let res = res
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
    /// Prefer this method over `get` if you only need a reference to the value
    /// and your backend supports it.
    fn get_ref<'k>(&'c self, key: &'k Key::EItem) -> Result<Option<RefValue<Ref, Val::DItem>>>
    where
        Key: BytesEncode<'k>,
        Val: BytesDecode<'c>;
}

/// A database that can return references in batches.
pub trait DBCommonRefBatch<'c, Key, Val, Ref>
where
    Ref: AsRef<[u8]> + 'c + std::ops::Deref<Target = [u8]> + Send + Sync,
{
    /// Get the serialized `val` from the database by `key`.
    /// Prefer this method over `get_multi` if you only need a reference to the value
    /// and your backend supports it.
    #[allow(clippy::type_complexity)] // trait associated type defaults are not stable yet
    fn get_multi_ref<'k, I>(&'c self, keys: I) -> Result<Vec<Option<RefValue<Ref, Val::DItem>>>>
    where
        Key: BytesEncode<'k>,
        I: IntoIterator<Item = &'k Key::EItem>,
        Val: BytesDecode<'c>;
}

/// A database that supports iterators.
pub trait DBCommonIter<'c, Key, Val, Iterator> {
    /// Get an iterator over the database.
    fn iter(&'c self) -> Result<Iterator>;
}

/// A database that supports iterators over a prefix.
pub trait DBCommonIterPrefix<'c, Key: 'c, Val: 'c> {
    /// The iterator type, must implement `Iterator<Item = Result<(Key::DItem, Val::DItem)>>`.
    type Iter: Iterator<Item = Result<(Vec<u8>, Vec<u8>)>>;

    /// Get a raw iterator over the database.
    fn iter_prefix_raw(&self, prefix: impl AsRef<[u8]>) -> Result<Self::Iter>;

    /// Get an iterator over the database.
    fn iter_prefix<'k, Prefix>(
        &self,
        prefix: &'k Prefix::EItem,
    ) -> Result<Box<dyn Iterator<Item = Result<(Key::DItem, Val::DItem)>> + 'c>>
    where
        Val: BytesDecodeOwned,
        Key: BytesDecodeOwned,
        Prefix: BytesEncode<'k>,
        Self: 'c,
    {
        let prefix_bytes = Prefix::bytes_encode(prefix)?;
        let raw = self.iter_prefix_raw(&prefix_bytes)?;

        let res = raw.map(|item| {
            let (key_bytes, val_bytes) = item?;
            let key = Key::bytes_decode_owned(&key_bytes)?;
            let val = Val::bytes_decode_owned(&val_bytes)?;
            Ok((key, val))
        });

        Ok(Box::new(res))
    }
}
