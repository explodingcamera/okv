use std::borrow::Cow;

use crate::Result;

mod any;

#[cfg(feature = "memdb")]
pub mod mem;

#[cfg(feature = "rocksdb")]
pub mod rocksdb;

pub trait DatabaseBackend<'d, 'c>
where
    Self: Sized,
{
    /// The type of the 'inner', this is a reference to the underlying database backend.
    type Inner;
    fn inner(&self) -> &Self::Inner;

    /// The type of the 'column', this is a reference to a database.
    type Column: DatabaseColumn<'c>;
    fn create_or_open(&'d self, db: &str) -> Result<Self::Column>;
}

pub trait DatabaseColumn<'c> {
    type Inner;
    fn inner(&self) -> &Self::Inner;

    fn set(&self, key: Cow<[u8]>, val: &[u8]) -> Result<()>;
    fn get(&self, key: Cow<[u8]>) -> Result<Option<Vec<u8>>>;
}

pub trait DatabaseColumnRef<'c>: DatabaseColumn<'c> {
    type Ref: AsRef<[u8]> + 'c + std::ops::Deref<Target = [u8]> + Send + Sync;
    fn get_ref(&self, key: Cow<[u8]>) -> Result<Option<Self::Ref>>;
}

pub trait DatabaseColumnTxn<'c>: DatabaseColumn<'c> {
    fn begin(&self) -> Result<()>;
    fn commit(&self) -> Result<()>;
    fn rollback(&self) -> Result<()>;
}
