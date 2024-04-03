use crate::{
    backend::DatabaseBackend, backend_async::DBColumnAsync, error::Result, traits::BytesEncode,
};

use super::Database;

impl<Key, Val, D: DatabaseBackend> crate::traits_async::DBCommonAsync<Key, Val>
    for Database<Key, Val, D>
where
    D::Column: DBColumnAsync,
{
    async fn aset<'k, 'v>(&'v self, key: &'k <Key>::EItem, val: &'v <Val>::EItem) -> Result<()>
    where
        Key: BytesEncode<'k>,
        Val: BytesEncode<'v>,
    {
        self.aset_raw(Key::bytes_encode(key)?, &Val::bytes_encode(val)?)
            .await
    }

    fn aset_raw<'v>(
        &'v self,
        key: impl AsRef<[u8]>,
        val: &'v [u8],
    ) -> impl std::future::Future<Output = Result<()>> + Send {
        self.column.async_set(key, val)
    }
}
