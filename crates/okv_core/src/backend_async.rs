use crate::{backend::DBColumn, error::Result};

pub trait DBColumnAsync {
    fn async_set(
        &self,
        key: impl AsRef<[u8]>,
        val: impl AsRef<[u8]>,
    ) -> impl std::future::Future<Output = Result<()>> + Send;
}

impl<T: DBColumnAsync> DBColumn for T {
    fn set(&self, key: impl AsRef<[u8]>, val: impl AsRef<[u8]>) -> Result<()> {
        todo!()
    }

    fn get(&self, key: impl AsRef<[u8]>) -> Result<Option<Vec<u8>>> {
        todo!()
    }

    fn get_multi<I>(&self, keys: I) -> Result<Vec<Option<Vec<u8>>>>
    where
        I: IntoIterator,
        I::Item: AsRef<[u8]>,
    {
        todo!()
    }

    fn delete(&self, key: impl AsRef<[u8]>) -> Result<()> {
        todo!()
    }

    fn contains(&self, key: impl AsRef<[u8]>) -> Result<bool> {
        todo!()
    }
}
