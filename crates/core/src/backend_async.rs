use crate::error::Result;
use std::future::Future;

pub trait DBColumnAsync {
    /// Set a key-value pair.
    fn async_set(
        &self,
        key: impl AsRef<[u8]>,
        val: impl AsRef<[u8]> + Send,
    ) -> impl Future<Output = Result<()>> + Send;

    /// Set a key-value pair if the key does not exist.
    fn async_set_nx<'a>(
        &self,
        key: impl AsRef<[u8]> + 'a,
        val: impl AsRef<[u8]> + 'a + Send,
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
            okv_core::async_fallback_impl!();
        }
    };
}

#[macro_export]
macro_rules! async_fallback_impl {
    () => {
        fn async_set(
            &self,
            key: impl AsRef<[u8]>,
            val: impl AsRef<[u8]>,
        ) -> impl std::future::Future<Output = okv_core::error::Result<()>> + Send + '_ {
            let res = self.set(key, val);
            async move { res }
        }

        fn async_get(
            &self,
            key: impl AsRef<[u8]>,
        ) -> impl std::future::Future<Output = okv_core::error::Result<Option<Vec<u8>>>> + Send + '_ {
            let res = self.get(key);
            async move { res }
        }

        fn async_get_multi<I>(
            &self,
            keys: I,
        ) -> impl std::future::Future<Output = okv_core::error::Result<Vec<Option<Vec<u8>>>>> + Send + '_
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
        ) -> impl std::future::Future<Output = okv_core::error::Result<()>> + Send + '_ {
            let res = self.delete(key);
            async move { res }
        }

        fn async_contains(
            &self,
            key: impl AsRef<[u8]>,
        ) -> impl std::future::Future<Output = okv_core::error::Result<bool>> + Send + '_ {
            let res = self.contains(key);
            async move { res }
        }
    };
}

#[cfg(feature = "async")]
#[macro_export]
macro_rules! sync_fallback_impl {
    () => {
        fn set(&self, key: impl AsRef<[u8]>, val: impl AsRef<[u8]>) -> okv_core::error::Result<()> {
            futures::executor::block_on(
                self.async_set(key.as_ref().to_vec(), val.as_ref().to_vec()),
            )
        }

        fn get(&self, key: impl AsRef<[u8]>) -> okv_core::error::Result<Option<Vec<u8>>> {
            futures::executor::block_on(self.async_get(key.as_ref().to_vec()))
        }

        fn get_multi<I>(&self, keys: I) -> okv_core::error::Result<Vec<Option<Vec<u8>>>>
        where
            I: IntoIterator,
            I::Item: AsRef<[u8]>,
        {
            futures::executor::block_on(self.async_get_multi(keys))
        }

        fn delete(&self, key: impl AsRef<[u8]>) -> okv_core::error::Result<()> {
            futures::executor::block_on(self.async_delete(key.as_ref().to_vec()))
        }

        fn contains(&self, key: impl AsRef<[u8]>) -> okv_core::error::Result<bool> {
            futures::executor::block_on(self.async_contains(key.as_ref().to_vec()))
        }
    };
}

#[cfg(feature = "async")]
#[macro_export]
macro_rules! sync_fallback {
    ( $column:ty ) => {
        impl okv_core::backend::DBColumn for $column
        where
            $column: okv_core::backend_async::DBColumnAsync,
        {
            okv_core::sync_fallback_impl!();
        }
    };
}

#[cfg(feature = "async")]
pub use sync_fallback;
#[cfg(feature = "async")]
pub use sync_fallback_impl;
