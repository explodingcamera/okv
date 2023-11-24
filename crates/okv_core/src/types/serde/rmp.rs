use std::borrow::Cow;

use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{
    traits::{BytesDecode, BytesDecodeOwned, BytesEncode},
    DecodeError, EncodeError,
};

/// Describes a type that is [`Serialize`]/[`Deserialize`] and uses `rmp_serde` to do so.
///
/// It can borrow bytes from the original slice.
pub struct SerdeRmp<T>(std::marker::PhantomData<T>);

impl<'a, T: 'a> BytesEncode<'a> for SerdeRmp<T>
where
    T: Serialize,
{
    type EItem = T;

    fn bytes_encode(item: &Self::EItem) -> Result<Cow<[u8]>, EncodeError> {
        rmp_serde::to_vec(item).map(Cow::Owned).map_err(Into::into)
    }
}

impl<'a, T: 'a> BytesDecode<'a> for SerdeRmp<T>
where
    T: Deserialize<'a>,
{
    type DItem = T;

    fn bytes_decode(bytes: &'a [u8]) -> Result<Self::DItem, DecodeError> {
        rmp_serde::from_slice(bytes).map_err(Into::into)
    }
}

impl<T> BytesDecodeOwned for SerdeRmp<T>
where
    T: DeserializeOwned,
{
    type DItem = T;

    fn bytes_decode_owned(bytes: &[u8]) -> Result<Self::DItem, DecodeError> {
        serde_json::from_slice(bytes).map_err(Into::into)
    }
}