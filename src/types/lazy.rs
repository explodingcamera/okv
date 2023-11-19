use std::marker;

use crate::{
    traits::{BytesDecode, BytesDecodeOwned},
    DecodeError,
};

/// Lazily decode the data bytes, it can be used to avoid CPU intensive decoding
/// before making sure we really need to decode it (e.g. based on the key).
#[derive(Default)]
pub struct LazyDecode<C>(marker::PhantomData<C>);

/// Owns bytes that can be decoded on demand.
#[derive(Copy, Clone)]
pub struct Lazy<'a, C> {
    data: &'a [u8],
    marker: marker::PhantomData<C>,
}

/// Owns bytes that can be decoded on demand.
#[derive(Clone)]
pub struct LazyVec<C> {
    data: Vec<u8>,
    marker: marker::PhantomData<C>,
}

impl<'a, C: 'static> BytesDecode<'a> for LazyDecode<C> {
    type DItem = Lazy<'a, C>;

    fn bytes_decode(bytes: &'a [u8]) -> Result<Self::DItem, DecodeError> {
        Ok(Lazy {
            data: bytes,
            marker: marker::PhantomData,
        })
    }
}

impl<'a, C: BytesDecode<'a>> Lazy<'a, C> {
    /// Decode the given bytes as DItem
    pub fn decode(&self) -> Result<C::DItem, DecodeError> {
        C::bytes_decode(self.data)
    }
}

impl<C: 'static> BytesDecodeOwned for LazyDecode<C> {
    type DItem = LazyVec<C>;

    fn bytes_decode_owned(bytes: &[u8]) -> Result<Self::DItem, DecodeError> {
        Ok(LazyVec {
            data: bytes.to_vec(),
            marker: marker::PhantomData,
        })
    }
}

impl<C: BytesDecodeOwned> LazyVec<C> {
    /// Decode the given bytes as DItem
    pub fn decode(&self) -> Result<C::DItem, DecodeError> {
        C::bytes_decode_owned(&self.data)
    }
}
