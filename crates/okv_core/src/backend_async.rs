use std::future::Future;

use crate::error::Result;

pub trait DBColumnAsync {
    /// Set a key-value pair.
    fn async_set(
        &self,
        key: impl AsRef<[u8]>,
        val: impl AsRef<[u8]>,
    ) -> impl Future<Output = Result<()>> + Send;

    /// Set a key-value pair if the key does not exist.
    fn async_set_nx<'a>(
        &self,
        key: impl AsRef<[u8]> + 'a,
        val: impl AsRef<[u8]> + 'a,
    ) -> impl Future<Output = Result<bool>> {
        async move {
            let key = key.as_ref();
            let val = val.as_ref();
            let contains = self.async_contains(key);
            if contains.await? {
                Ok(false)
            } else {
                self.async_set(key, val).await?;
                Ok(true)
            }
        }
    }

    /// Get a value by key.
    fn async_get(
        &self,
        key: impl AsRef<[u8]>,
    ) -> impl Future<Output = Result<Option<Vec<u8>>>> + Send;

    /// Get a value by key in batch.
    fn async_get_multi<I>(
        &self,
        keys: I,
    ) -> impl Future<Output = Result<Vec<Option<Vec<u8>>>>> + Send
    where
        I: IntoIterator,
        I::Item: AsRef<[u8]>;

    /// Delete a key-value pair.
    fn async_delete(&self, key: impl AsRef<[u8]>) -> impl Future<Output = Result<()>> + Send;

    /// Check if a key exists.
    fn async_contains(&self, key: impl AsRef<[u8]>) -> impl Future<Output = Result<bool>> + Send;
}

// implement async methods for a struct that implements DBColumn
// Using the existing sync methods is recommended.
#[macro_export]
macro_rules! async_fallback {
    ( $column:ty ) => {
        impl okv_core::backend_async::DBColumnAsync for $column
        where
            $column: okv_core::backend::DBColumn,
        {
            fn async_set(
                &self,
                key: impl AsRef<[u8]>,
                val: impl AsRef<[u8]>,
            ) -> impl std::future::Future<Output = Result<()>> + Send + '_ {
                let res = self.set(key, val);
                async move { res }
            }

            fn async_get(
                &self,
                key: impl AsRef<[u8]>,
            ) -> impl std::future::Future<Output = Result<Option<Vec<u8>>>> + Send + '_ {
                let res = self.get(key);
                async move { res }
            }

            fn async_get_multi<I>(
                &self,
                keys: I,
            ) -> impl std::future::Future<Output = Result<Vec<Option<Vec<u8>>>>> + Send + '_
            where
                I: IntoIterator,
                I::Item: AsRef<[u8]>,
            {
                let res = self.get_multi(keys);
                async move { res }
            }

            fn async_delete(
                &self,
                key: impl AsRef<[u8]>,
            ) -> impl std::future::Future<Output = Result<()>> + Send + '_ {
                let res = self.delete(key);
                async move { res }
            }

            fn async_contains(
                &self,
                key: impl AsRef<[u8]>,
            ) -> impl std::future::Future<Output = Result<bool>> + Send + '_ {
                let res = self.contains(key);
                async move { res }
            }
        }
    };
}

pub use async_fallback;
