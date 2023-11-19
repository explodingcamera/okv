use std::borrow::Cow;

use crate::Result;

pub mod mem;

pub trait DatabaseBackend<'d, 'c> {
    type Config: Clone;
    type Column: DatabaseColumn<'c>;

    fn new(connect_str: &str) -> Self;
    fn new_with_config(config: Self::Config) -> Self;
    fn create_or_open(&'d self, db: &str) -> Result<Self::Column>;
}

pub trait DatabaseColumn<'c> {
    fn set(&self, key: Cow<[u8]>, val: &[u8]) -> Result<()>;
    fn get(&self, key: Cow<[u8]>) -> Option<Vec<u8>>;
}

pub trait DatabaseColumnRef<'c>: DatabaseColumn<'c> {
    fn get_ref(&self, key: Cow<[u8]>) -> Option<&[u8]>;
}

pub trait DatabaseColumnTxn<'c>: DatabaseColumn<'c> {
    fn begin(&self) -> Result<()>;
    fn commit(&self) -> Result<()>;
    fn rollback(&self) -> Result<()>;
}
