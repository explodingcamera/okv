use std::future::Future;

use crate::{
    error::{EncodeError, Result},
    traits::{BytesDecodeOwned, BytesEncode},
};

pub trait DBCommonAsync<Key, Val> {
    /// Set a key to a value in the database.
    fn aset_raw<'v>(
        &'v self,
        key: impl AsRef<[u8]>,
        val: &'v [u8],
    ) -> impl Future<Output = Result<()>> + Send;

    /// Set a `key` to a value in the database if the key does not exist.
    fn aset_nx_raw<'v>(
        &'v self,
        key: impl AsRef<[u8]>,
        val: &'v [u8],
    ) -> impl Future<Output = Result<bool>> + Send;

    /// Delete the serialized `val` from the database by `key`.
    fn adelete<'k>(&self, key: &'k Key::EItem) -> impl Future<Output = Result<()>>
    where
        Key: BytesEncode<'k>;

    /// Check if the database contains the given key.
    fn acontains<'k>(&self, key: &'k Key::EItem) -> impl Future<Output = Result<bool>>
    where
        Key: BytesEncode<'k>;

    /// Set a `key` to the serialized `val` in the database.
    fn aset<'k, 'v>(
        &'v self,
        key: &'k Key::EItem,
        val: &'v Val::EItem,
    ) -> impl Future<Output = Result<()>>
    where
        Key: BytesEncode<'k>,
        Val: BytesEncode<'v>,
    {
        let key = Key::bytes_encode(key);
        let val = Val::bytes_encode(val);
        async { self.aset_raw(key?, &val?).await }
    }

    /// Set a `key` to a serialized value in the database if the key does not exist.
    fn aset_nx<'k, 'v>(
        &'v self,
        key: &'k Key::EItem,
        val: &'v Val::EItem,
    ) -> impl Future<Output = Result<bool>>
    where
        Key: BytesEncode<'k>,
        Val: BytesEncode<'v>,
    {
        async {
            self.aset_nx_raw(Key::bytes_encode(key)?, &Val::bytes_encode(val)?)
                .await
        }
    }

    /// Set a `key` to a value in the database.
    fn aget_raw(
        &self,
        key: impl AsRef<[u8]>,
    ) -> impl Future<Output = Result<Option<Vec<u8>>>> + Send;

    /// Get the serialized `val` from the database by `key`.
    fn aget<'k, 'v>(&self, key: &'k Key::EItem) -> impl Future<Output = Result<Option<Val::DItem>>>
    where
        Key: BytesEncode<'k>,
        Val: BytesDecodeOwned,
    {
        async {
            let key_bytes = Key::bytes_encode(key)?;
            let val_bytes = self.aget_raw(key_bytes).await?;
            match val_bytes {
                Some(val_bytes) => Ok(Some(Val::bytes_decode_owned(&val_bytes)?)),
                None => Ok(None),
            }
        }
    }

    /// Get values from the database by `keys`.
    fn aget_multi_raw<I, IV: AsRef<[u8]>>(
        &self,
        keys: I,
    ) -> impl Future<Output = Result<Vec<Option<Vec<u8>>>>> + Send
    where
        I: IntoIterator<Item = IV>;

    /// Get the serialized `val` from the database by `key`.
    fn aget_multi<'k, I>(&self, keys: I) -> impl Future<Output = Result<Vec<Option<Val::DItem>>>>
    where
        Key: BytesEncode<'k>,
        I: IntoIterator<Item = &'k Key::EItem>,
        Val: BytesDecodeOwned,
    {
        let encoded_keys: Result<Vec<Vec<u8>>, EncodeError> = keys
            .into_iter()
            .map(|key| Key::bytes_encode(key).map(|cow| cow.into_owned()))
            .collect();

        async {
            let res = self
                .aget_multi_raw(encoded_keys?)
                .await?
                .iter()
                .map(|item| match item {
                    Some(val_bytes) => Ok(Some(Val::bytes_decode_owned(val_bytes)?)),
                    None => Ok(None),
                })
                .collect::<Result<Vec<Option<Val::DItem>>>>()?;

            Ok(res)
        }
    }
}
