use crate::error::Result;

pub trait DBColumnAsync {
    fn async_set(
        &self,
        key: impl AsRef<[u8]>,
        val: impl AsRef<[u8]>,
    ) -> impl std::future::Future<Output = Result<()>> + Send;
}

// implement async methods for a struct that implements DBColumn
// Using the existing sync methods is recommended.
#[macro_export]
macro_rules! async_fallback {
    ( $column:ty ) => {
        #[inherent::inherent]
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
        }
    };
}

pub use async_fallback;
