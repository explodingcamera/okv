use dashmap::{try_result::TryResult, DashMap};
use okv_core::{
    backend::*,
    env::Env,
    error::{Error, Result},
    traits::*,
};
use self_cell::self_cell;

/// An in-memory database backend.
/// This is useful for testing and prototyping.
/// Not optimized for performance.
#[derive(Clone)]
pub struct MemDB {
    columns: DashMap<String, DashMap<Vec<u8>, Vec<u8>>>,
}

impl MemDB {
    /// Create a new in-memory database backend.
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for MemDB {
    fn default() -> Self {
        Self {
            columns: DashMap::new(),
        }
    }
}

impl Innerable for MemDB {
    type Inner = DashMap<String, DashMap<Vec<u8>, Vec<u8>>>;
    fn inner(&self) -> &Self::Inner {
        &self.columns
    }
}

impl DatabaseBackend for MemDB {
    type Column = MemDBColumn;

    fn create_or_open(env: Env<MemDB>, name: &str) -> Result<Self::Column> {
        let col = MemDBColumn::try_new(env.clone(), |backend| {
            match backend.db().columns.try_get(name) {
                TryResult::Absent => {
                    let col = DashMap::new();
                    backend.db().columns.insert(name.to_owned(), col);
                    let col = backend
                        .db()
                        .columns
                        .try_get(name)
                        .try_unwrap()
                        .expect("Newly inserted column should not be locked");
                    Ok(col)
                }
                TryResult::Present(col) => Ok(col),
                TryResult::Locked => Err(Error::DatabaseNotFound {
                    db: name.to_string(),
                }),
            }
        })?;

        Ok(col)
    }
}

type MemDBColumnInner<'a> = dashmap::mapref::one::Ref<'a, String, DashMap<Vec<u8>, Vec<u8>>>;

self_cell!(
    /// A column in an in-memory database.
    pub struct MemDBColumn {
        // _name: String,
        owner: Env<MemDB>,

        #[covariant]
        dependent: MemDBColumnInner,
    }
);

impl DBColumnClear for MemDBColumn {
    fn clear(&self) -> Result<()> {
        self.borrow_dependent().clear();
        Ok(())
    }
}

impl Flushable for MemDBColumn {
    /// No-op.
    fn flush(&self) -> Result<()> {
        Ok(())
    }
}

impl DBColumn for MemDBColumn {
    fn set(&self, key: impl AsRef<[u8]>, val: impl AsRef<[u8]>) -> Result<()> {
        self.borrow_dependent()
            .insert(key.as_ref().to_vec(), val.as_ref().to_vec());
        Ok(())
    }

    fn get(&self, key: impl AsRef<[u8]>) -> Result<Option<Vec<u8>>> {
        match self.borrow_dependent().get(&key.as_ref().to_vec()) {
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
        Ok(self.borrow_dependent().contains_key(&key.as_ref().to_vec()))
    }

    fn delete(&self, key: impl AsRef<[u8]>) -> Result<()> {
        self.borrow_dependent().remove(&key.as_ref().to_vec());
        Ok(())
    }
}

impl DBColumnIterator for MemDBColumn {
    fn iter(&self) -> Result<impl Iterator<Item = Result<(Vec<u8>, Vec<u8>)>>> {
        Ok(self
            .borrow_dependent()
            .iter()
            .map(|item| (item.key().clone(), item.value().clone()))
            .map(Ok))
    }
}

impl DBColumnIteratorPrefix for MemDBColumn {
    fn iter_prefix(
        &self,
        prefix: impl AsRef<[u8]>,
    ) -> Result<impl Iterator<Item = Result<(Vec<u8>, Vec<u8>)>>> {
        let prefix = prefix.as_ref().to_vec();
        let iter = self
            .borrow_dependent()
            .iter()
            .filter(move |item| item.key().starts_with(&prefix))
            .map(|item| (item.key().clone(), item.value().clone()))
            .map(Ok);

        Ok(iter)
    }
}
