use std::borrow::Cow;

use crate::{
    traits::{BytesDecode, BytesDecodeOwned, BytesEncode},
    DecodeError, EncodeError,
};

impl BytesEncode<'_> for &str {
    type EItem = str;

    fn bytes_encode(item: &Self::EItem) -> Result<Cow<[u8]>, EncodeError> {
        Ok(Cow::Borrowed(item.as_bytes()))
    }
}

impl BytesEncode<'_> for String {
    type EItem = str;

    fn bytes_encode(item: &Self::EItem) -> Result<Cow<[u8]>, EncodeError> {
        Ok(Cow::Borrowed(item.as_bytes()))
    }
}

impl<'a> BytesDecode<'a> for &'a str {
    type DItem = &'a str;

    fn bytes_decode(bytes: &'a [u8]) -> Result<Self::DItem, DecodeError> {
        std::str::from_utf8(bytes).map_err(|_| DecodeError::InvalidUtf8)
    }
}

impl BytesDecodeOwned for &str {
    type DItem = String;

    fn bytes_decode_owned(bytes: &[u8]) -> Result<Self::DItem, DecodeError> {
        match std::str::from_utf8(bytes) {
            Ok(s) => Ok(s.to_owned()),
            Err(_) => Err(DecodeError::InvalidUtf8),
        }
    }
}

impl BytesDecodeOwned for String {
    type DItem = String;

    fn bytes_decode_owned(bytes: &[u8]) -> Result<Self::DItem, DecodeError> {
        match std::str::from_utf8(bytes) {
            Ok(s) => Ok(s.to_owned()),
            Err(_) => Err(DecodeError::InvalidUtf8),
        }
    }
}
