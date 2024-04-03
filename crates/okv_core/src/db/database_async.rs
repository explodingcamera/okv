use super::Database;
use std::future::Future;

use crate::backend::DatabaseBackend;
use crate::backend_async::DBColumnAsync;
use crate::{error::Result, traits::BytesEncode};

#[allow(clippy::manual_async_fn)]
impl<Key, Val, D: DatabaseBackend> crate::traits_async::DBCommonAsync<Key, Val>
    for Database<Key, Val, D>
where
    D::Column: DBColumnAsync + Send + Sync,
    Key: Send + Sync,
    Val: Send + Sync,
{
    fn aset<'k, 'v>(
        &'v self,
        key: &'k <Key>::EItem,
        val: &'v <Val>::EItem,
    ) -> impl Future<Output = Result<()>> + Send
    where
        Key: BytesEncode<'k>,
        Val: BytesEncode<'v>,
    {
        let key = Key::bytes_encode(key);
        let val = Val::bytes_encode(val);

        async { self.aset_raw(key?, &val?).await }
    }

    fn aset_raw<'v>(
        &'v self,
        key: impl AsRef<[u8]> + Send + Sync,
        val: &'v [u8],
    ) -> impl Future<Output = Result<()>> + Send {
        self.column.async_set(key, val)
    }

    fn aset_nx_raw<'v>(
        &'v self,
        _key: impl AsRef<[u8]>,
        _val: &'v [u8],
    ) -> impl Future<Output = Result<bool>> + Send {
        async { todo!() }
    }

    fn adelete<'k>(&self, _key: &'k <Key>::EItem) -> impl Future<Output = Result<()>> + Send
    where
        Key: BytesEncode<'k>,
    {
        async { todo!() }
    }

    fn acontains<'k>(&self, _key: &'k <Key>::EItem) -> impl Future<Output = Result<bool>> + Send
    where
        Key: BytesEncode<'k>,
    {
        async { todo!() }
    }

    fn aget_raw(
        &self,
        _key: impl AsRef<[u8]>,
    ) -> impl Future<Output = Result<Option<Vec<u8>>>> + Send {
        async { todo!() }
    }

    fn aget_multi_raw<I, IV: AsRef<[u8]>>(
        &self,
        _keys: I,
    ) -> impl Future<Output = Result<Vec<Option<Vec<u8>>>>> + Send
    where
        I: IntoIterator<Item = IV>,
    {
        async { todo!() }
    }
}
