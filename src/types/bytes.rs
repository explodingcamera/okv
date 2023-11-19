use std::borrow::Cow;

use crate::{
    traits::{BytesDecode, BytesDecodeOwned, BytesEncode},
    DecodeError, EncodeError,
};

impl BytesEncode<'_> for &[u8] {
    type EItem = [u8];

    fn bytes_encode(item: &Self::EItem) -> Result<Cow<[u8]>, EncodeError> {
        Ok(Cow::Borrowed(item))
    }
}

impl<const L: usize> BytesDecode<'_> for [u8; L] {
    type DItem = [u8; L];

    fn bytes_decode(bytes: &[u8]) -> Result<Self::DItem, DecodeError> {
        bytes
            .len()
            .eq(&L)
            .then(|| {
                let mut arr = [0u8; L];
                arr.copy_from_slice(bytes);
                arr
            })
            .ok_or(DecodeError::SizeMismatch)
    }
}

impl BytesEncode<'_> for Vec<u8> {
    type EItem = [u8];

    fn bytes_encode(item: &Self::EItem) -> Result<Cow<[u8]>, EncodeError> {
        Ok(Cow::Owned(item.to_vec()))
    }
}

impl<'a> BytesDecode<'a> for &[u8] {
    type DItem = Cow<'a, [u8]>;

    fn bytes_decode(bytes: &'a [u8]) -> Result<Self::DItem, DecodeError> {
        Ok(Cow::Borrowed(bytes))
    }
}

impl BytesDecodeOwned for &[u8] {
    type DItem = Vec<u8>;

    fn bytes_decode_owned(bytes: &[u8]) -> Result<Self::DItem, DecodeError> {
        Ok(bytes.to_vec())
    }
}

impl BytesDecodeOwned for Vec<u8> {
    type DItem = Vec<u8>;

    fn bytes_decode_owned(bytes: &[u8]) -> Result<Self::DItem, DecodeError> {
        Ok(bytes.to_vec())
    }
}
