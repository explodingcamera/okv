use std::borrow::Cow;
use std::mem::size_of;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::error::{DecodeError, EncodeError};
use crate::traits::{BytesDecode, BytesDecodeOwned, BytesEncode};

impl BytesEncode<'_> for u8 {
    type EItem = u8;

    fn bytes_encode(item: &Self::EItem) -> Result<Cow<[u8]>, EncodeError> {
        Ok(Cow::from([*item].to_vec()))
    }
}

impl BytesDecode<'_> for u8 {
    type DItem = u8;

    fn bytes_decode(mut bytes: &'_ [u8]) -> Result<Self::DItem, DecodeError> {
        bytes.read_u8().map_err(Into::into)
    }
}

impl BytesDecodeOwned for u8 {
    type DItem = u8;

    fn bytes_decode_owned(mut bytes: &[u8]) -> Result<Self::DItem, DecodeError> {
        bytes.read_u8().map_err(Into::into)
    }
}

macro_rules! define_type {
    ($name:ident, $read_method:ident, $write_method:ident) => {
        #[doc = "Encodable version of [`"]
        #[doc = stringify!($name)]
        #[doc = "`] (Little Endian)."]

        impl BytesEncode<'_> for $name {
            type EItem = $name;

            fn bytes_encode(item: &Self::EItem) -> Result<Cow<[u8]>, EncodeError> {
                let mut buf = vec![0; size_of::<Self::EItem>()];
                buf.$write_method::<LittleEndian>(*item)
                    .map_err(EncodeError::from)?;
                Ok(Cow::from(buf))
            }
        }

        impl BytesDecode<'_> for $name {
            type DItem = $name;

            fn bytes_decode(mut bytes: &'_ [u8]) -> Result<Self::DItem, DecodeError> {
                bytes.$read_method::<LittleEndian>().map_err(Into::into)
            }
        }

        impl BytesDecodeOwned for $name {
            type DItem = $name;

            fn bytes_decode_owned(mut bytes: &[u8]) -> Result<Self::DItem, DecodeError> {
                bytes.$read_method::<LittleEndian>().map_err(Into::into)
            }
        }
    };
}

define_type!(u16, read_u16, write_u16);
define_type!(u32, read_u32, write_u32);
define_type!(u64, read_u64, write_u64);
define_type!(u128, read_u128, write_u128);

define_type!(i16, read_i16, write_i16);
define_type!(i32, read_i32, write_i32);
define_type!(i64, read_i64, write_i64);
define_type!(i128, read_i128, write_i128);
