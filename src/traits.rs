use std::borrow::Cow;

use crate::{types::RefValue, DecodeError, EncodeError, Result};

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
    /// Set a `key` to the serialized `val` in the database.
    fn set<'k, 'v>(&'v mut self, key: &'k Key::EItem, val: &'v Val::EItem) -> Result<()>
    where
        Key: BytesEncode<'k>,
        Val: BytesEncode<'v>;

    /// Get the serialized `val` from the database by `key`.
    fn get<'k, 'v>(&self, key: &'k Key::EItem) -> Result<Val::DItem>
    where
        Key: BytesEncode<'k>,
        Val: BytesDecodeOwned;

    /// Get the serialized `val` from the database by `key`.
    fn get_multi<'k, I>(&self, keys: I) -> Result<Vec<Option<Val::DItem>>>
    where
        Key: BytesEncode<'k>,
        I: IntoIterator<Item = &'k Key::EItem>,
        Val: BytesDecodeOwned;

    /// Delete the serialized `val` from the database by `key`.
    fn delete<'k>(&mut self, key: &'k Key::EItem) -> Result<()>
    where
        Key: BytesEncode<'k>;

    /// Clear the database, removing all key-value pairs.
    fn clear(&mut self) -> Result<()>;

    /// Check if the database contains the given key.
    fn contains<'k>(&self, key: &'k Key::EItem) -> Result<bool>
    where
        Key: BytesEncode<'k>;
}

/// A trait that represents a common database interface that returns references.
pub trait DBCommonRef<'c, Key, Val, Ref>
where
    Ref: AsRef<[u8]> + 'c + std::ops::Deref<Target = [u8]> + Send + Sync,
{
    /// Get the serialized `val` from the database by `key`.
    /// Prefer this method over `get` if you only need a reference to the value
    /// and your backend supports it.
    fn get_ref<'k, 'v>(&'v self, key: &'k Key::EItem) -> Result<RefValue<Ref, Val::DItem>>
    where
        Key: BytesEncode<'k>,
        Val: BytesDecode<'c>;

    /// Get the serialized `val` from the database by `key`.
    /// Prefer this method over `get_multi` if you only need a reference to the value
    /// and your backend supports it.
    #[allow(clippy::type_complexity)] // trait associated type defaults are not stable yet
    fn get_multi_ref<'k, I>(&self, keys: I) -> Result<Vec<Option<RefValue<Ref, Val::DItem>>>>
    where
        Key: BytesEncode<'k>,
        I: IntoIterator<Item = &'k Key::EItem>,
        Val: BytesDecode<'c>;
}
