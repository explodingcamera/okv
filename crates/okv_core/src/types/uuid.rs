use std::borrow::Cow;

use uuid::Uuid;

use crate::error::{DecodeError, EncodeError};
use crate::traits::{BytesDecode, BytesDecodeOwned, BytesEncode};

impl BytesEncode<'_> for Uuid {
    type EItem = Uuid;

    fn bytes_encode(item: &Self::EItem) -> Result<Cow<[u8]>, EncodeError> {
        Ok(Cow::Borrowed(item.as_bytes()))
    }
}

impl BytesDecode<'_> for Uuid {
    type DItem = Uuid;

    fn bytes_decode(bytes: &[u8]) -> Result<Self::DItem, DecodeError> {
        Ok(Uuid::from_slice(bytes)?)
    }
}

impl BytesDecodeOwned for Uuid {
    type DItem = Uuid;

    fn bytes_decode_owned(bytes: &[u8]) -> Result<Self::DItem, DecodeError> {
        Ok(Uuid::from_slice(bytes)?)
    }
}
