/// Serialization for bytes
pub mod bytes;

/// Lazy serialization
pub mod lazy;

/// Serialization for primitive types
pub mod primitive;

#[cfg(feature = "serde")]
/// Serialization for serde types (requires `serde` feature)
pub mod serde;

#[cfg(feature = "uuid")]
/// Serialization for cuid2 types (requires `cuid2` feature)
pub mod uuid;

/// A reference to a value in the database.
/// Allows for more efficient access to the underlying bytes by returning a reference.
/// To deserialize the value, use [`crate::types::RefValue::deserialize()`].
pub struct RefValue<'a, T, Val> {
    pub(crate) data: T,
    pub(crate) marker: std::marker::PhantomData<&'a Val>,
}

impl<'a, T, Val> RefValue<'a, T, Val>
where
    T: AsRef<[u8]> + 'a + std::ops::Deref<Target = [u8]> + Send + Sync,
    Val: crate::traits::BytesDecode<'a>,
{
    /// Returns a reference to the inner value.
    pub fn inner(&self) -> &T {
        &self.data
    }

    /// Deserialize the value from the database to the type `Val`.
    pub fn deserialize(&'a self) -> crate::error::Result<Val::DItem> {
        Ok(Val::bytes_decode(&self.data)?)
    }
}
