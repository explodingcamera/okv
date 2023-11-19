use std::borrow::Cow;

use crate::{
    traits::{BytesDecode, BytesDecodeOwned, BytesEncode},
    DecodeError, EncodeError,
};

impl BytesEncode<'static> for () {
    type EItem = ();

    fn bytes_encode(_item: &Self::EItem) -> Result<Cow<[u8]>, EncodeError> {
        Ok(Cow::Borrowed(&[]))
    }
}

impl BytesDecode<'static> for () {
    type DItem = ();

    fn bytes_decode(bytes: &[u8]) -> Result<Self::DItem, DecodeError> {
        bytes
            .is_empty()
            .then(|| ())
            .ok_or(DecodeError::SizeMismatch)
    }
}

impl BytesDecodeOwned for () {
    type DItem = ();

    fn bytes_decode_owned(bytes: &[u8]) -> Result<Self::DItem, DecodeError> {
        bytes
            .is_empty()
            .then(|| ())
            .ok_or(DecodeError::SizeMismatch)
    }
}
