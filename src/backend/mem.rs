use crate::{Error, Flushable, Result};

use super::{DBColumn, DBColumnClear, DatabaseBackend, Innerable};
use dashmap::{try_result::TryResult, DashMap};

/// An in-memory database backend.
/// This is useful for testing and prototyping.
/// Not optimized for performance.
#[derive(Clone)]
pub struct MemDB<'c> {
    columns: DashMap<String, DashMap<Vec<u8>, Vec<u8>>>,
    marker: std::marker::PhantomData<&'c ()>,
}

impl MemDB<'_> {
    /// Create a new in-memory database backend.
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

impl<'a> Innerable for MemDB<'a> {
    type Inner = DashMap<String, DashMap<Vec<u8>, Vec<u8>>>;
    fn inner(&self) -> &Self::Inner {
        &self.columns
    }
}

impl<'a> DatabaseBackend<'a> for MemDB<'a> {
    type Column = MemDBColumn<'a>;

    fn create_or_open(&'a self, name: &str) -> super::Result<Self::Column> {
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

        Ok(MemDBColumn { column: col })
    }
}

/// A column in an in-memory database.
pub struct MemDBColumn<'a> {
    // _name: String,
    // db: &'a MemDB<'a>,
    column: dashmap::mapref::one::Ref<'a, String, DashMap<Vec<u8>, Vec<u8>>>,
}

impl<'a> Innerable for MemDBColumn<'a> {
    type Inner = dashmap::mapref::one::Ref<'a, String, DashMap<Vec<u8>, Vec<u8>>>;

    fn inner(&self) -> &Self::Inner {
        &self.column
    }
}

impl<'a> DBColumnClear for MemDBColumn<'a> {
    fn clear(&self) -> super::Result<()> {
        self.column.clear();
        Ok(())
    }
}

impl<'a> Flushable for MemDBColumn<'a> {
    /// No-op.
    fn flush(&self) -> super::Result<()> {
        Ok(())
    }
}

impl<'a> DBColumn for MemDBColumn<'a> {
    fn set(&self, key: impl AsRef<[u8]>, val: impl AsRef<[u8]>) -> super::Result<()> {
        self.column
            .insert(key.as_ref().to_vec(), val.as_ref().to_vec());
        Ok(())
    }

    fn get(&self, key: impl AsRef<[u8]>) -> Result<Option<Vec<u8>>> {
        match self.column.get(&key.as_ref().to_vec()) {
            None => Ok(None),
            Some(val) => Ok(Some(val.value().clone())),
        }
    }

    fn get_multi<I>(&self, keys: I) -> Result<Vec<Option<Vec<u8>>>>
    where
        I: IntoIterator,
        I::Item: AsRef<[u8]>,
    {
        let mut res = Vec::new();
        for key in keys {
            res.push(self.get(key)?);
        }
        Ok(res)
    }

    fn contains(&self, key: impl AsRef<[u8]>) -> Result<bool> {
        Ok(self.column.contains_key(&key.as_ref().to_vec()))
    }

    fn delete(&self, key: impl AsRef<[u8]>) -> Result<()> {
        self.column.remove(&key.as_ref().to_vec());
        Ok(())
    }
}
