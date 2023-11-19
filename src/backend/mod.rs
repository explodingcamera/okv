use crate::Result;

pub mod mem;

pub trait DatabaseBackend {
    fn create_or_open(&self, db: &str) -> Result<()>;

    fn set(&self, db: &str, key: &[u8], val: &[u8]) -> Result<()>;
    fn get(&self, db: &str, key: &[u8]) -> Option<Vec<u8>>;
}

pub trait DatabaseBackendRef: DatabaseBackend {
    fn get_ref(&self, db: &str, key: &[u8]) -> Option<&[u8]>;
}

pub trait DatabaseBackendTxn: DatabaseBackend {
    fn begin(&self) -> Result<()>;
    fn commit(&self) -> Result<()>;
    fn rollback(&self) -> Result<()>;
}
