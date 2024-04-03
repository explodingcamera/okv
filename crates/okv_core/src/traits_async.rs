use crate::{error::Result, traits::BytesEncode};

pub trait DBCommonAsync<Key, Val> {
    /// Get the serialized `val` from the database by `key`.
    fn aset<'k, 'v>(
        &'v self,
        key: &'k Key::EItem,
        val: &'v Val::EItem,
    ) -> impl std::future::Future<Output = Result<()>>
    where
        Key: BytesEncode<'k>,
        Val: BytesEncode<'v>,
    {
        let key = Key::bytes_encode(key);
        let val = Val::bytes_encode(val);
        async { self.aset_raw(key?, &val?).await }
    }

    /// Set a key to a value in the database.
    fn aset_raw<'v>(
        &'v self,
        key: impl AsRef<[u8]>,
        val: &'v [u8],
    ) -> impl std::future::Future<Output = Result<()>> + Send;
}
