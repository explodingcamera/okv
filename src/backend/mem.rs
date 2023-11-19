use std::borrow::Cow;

use crate::{Error, Result};

use super::{DatabaseBackend, DatabaseColumn};
use dashmap::{try_result::TryResult, DashMap};

#[derive(Clone)]
pub struct MemDB<'c> {
    columns: DashMap<String, DashMap<Vec<u8>, Vec<u8>>>,
    marker: std::marker::PhantomData<&'c ()>,
}

impl MemDB<'_> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for MemDB<'_> {
    fn default() -> Self {
        Self {
            columns: DashMap::new(),
            marker: std::marker::PhantomData,
        }
    }
}

impl<'b, 'c> DatabaseBackend<'b, 'c> for MemDB<'b>
where
    'b: 'c,
    Self: Sized,
{
    type Column = MemDBColumn<'c>;

    fn create_or_open(&'b self, name: &str) -> super::Result<Self::Column> {
        let col = match self.columns.try_get(name) {
            TryResult::Absent => {
                let col = DashMap::new();
                self.columns.insert(name.to_owned(), col);
                self.columns
                    .try_get(name)
                    .try_unwrap()
                    .expect("Newly inserted column should not be locked")
            }
            TryResult::Present(col) => col,
            TryResult::Locked => {
                return Err(Error::DatabaseNotFound {
                    db: name.to_string(),
                });
            }
        };

        return Ok(MemDBColumn {
            _name: name.to_owned(),
            db: self,
            column: col,
        });
    }
}

pub struct MemDBColumn<'a> {
    _name: String,
    db: &'a MemDB<'a>,
    column: dashmap::mapref::one::Ref<'a, String, DashMap<Vec<u8>, Vec<u8>>>,
}

impl<'a, 'c> DatabaseColumn<'c> for MemDBColumn<'a> {
    fn set(&self, key: Cow<[u8]>, val: &[u8]) -> super::Result<()> {
        self.column.insert(key.to_vec(), val.to_vec());
        Ok(())
    }

    fn get(&self, key: Cow<[u8]>) -> Result<Option<Vec<u8>>> {
        match self.column.get(&key.to_vec()) {
            None => Ok(None),
            Some(val) => Ok(Some(val.value().clone())),
        }
    }
}
