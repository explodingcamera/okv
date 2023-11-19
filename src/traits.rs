use std::borrow::Cow;

use crate::{DecodeError, EncodeError};

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
    // type DItemOwned;

    /// Decode the given bytes as DItem
    fn bytes_decode(bytes: &'a [u8]) -> Result<Self::DItem, DecodeError>;
    // Decode the given bytes as DItem
    // fn bytes_decode_owned(bytes: &[u8]) -> Result<Self::DItemOwned, DecodeError>;
}

pub trait BytesDecodeOwned {
    type DItem;
    fn bytes_decode_owned(bytes: &[u8]) -> Result<Self::DItem, DecodeError>;
}
