use okv_core::{
    backend::{DBColumn, DatabaseBackend},
    error::{Error, Result},
    traits::Innerable,
};

pub use redb;
use redb::{Database, ReadableTable, TableDefinition};
use self_cell::self_cell;

mod tx;

pub(crate) fn okv_err(e: impl Into<redb::Error>) -> Error {
    Error::DatabaseBackend(Box::new(e.into()))
}

pub struct Redb {
    db: Database,
}

impl Redb {
    pub fn new(connect_str: &str) -> Result<Self, redb::DatabaseError> {
        let db = Database::create(connect_str)?;
        Ok(Self { db })
    }
}

impl Innerable for Redb {
    type Inner = Database;
    fn inner(&self) -> &Self::Inner {
        &self.db
    }
}

pub struct RedbColumn {
    pub(crate) env: okv_core::env::Env<Redb>,
    pub(crate) table: RedbTableInner,
}

impl RedbColumn {
    fn db(&self) -> &Database {
        self.env.inner()
    }

    fn table(&self) -> TableDefinition<'_, &'static [u8], &'static [u8]> {
        self.table.borrow_dependent().0
    }
}

impl DBColumn for RedbColumn {
    fn contains(&self, key: impl AsRef<[u8]>) -> okv_core::error::Result<bool> {
        let tx = self.db().begin_read().map_err(okv_err)?;
        let table = tx.open_table(self.table()).map_err(okv_err)?;
        let res = table.get(key.as_ref()).map_err(okv_err)?;
        Ok(res.is_some())
    }

    fn delete(&self, key: impl AsRef<[u8]>) -> Result<()> {
        let tx = self.db().begin_write().map_err(okv_err)?;

        {
            let mut table = tx.open_table(self.table()).map_err(okv_err)?;
            table.remove(key.as_ref()).map_err(okv_err)?;
        }

        tx.commit().map_err(okv_err)?;
        Ok(())
    }

    fn get(&self, key: impl AsRef<[u8]>) -> Result<Option<Vec<u8>>> {
        let tx = self.db().begin_read().map_err(okv_err)?;
        let table = tx.open_table(self.table()).map_err(okv_err)?;
        let res = table.get(key.as_ref()).map_err(okv_err)?;
        Ok(res.map(|v| v.value().to_vec()))
    }

    fn get_multi<I>(&self, keys: I) -> Result<Vec<Option<Vec<u8>>>>
    where
        I: IntoIterator,
        I::Item: AsRef<[u8]>,
    {
        let mut res = Vec::new();
        let tx = self.db().begin_read().map_err(okv_err)?;
        let table = tx.open_table(self.table()).map_err(okv_err)?;
        for key in keys {
            let val = table.get(key.as_ref()).map_err(okv_err)?;
            res.push(val.map(|v| v.value().to_vec()));
        }

        Ok(res)
    }

    fn set(&self, key: impl AsRef<[u8]>, val: impl AsRef<[u8]>) -> Result<()> {
        let tx = self.db().begin_write().map_err(okv_err)?;

        {
            let mut table = tx.open_table(self.table()).map_err(okv_err)?;
            table.insert(key.as_ref(), val.as_ref()).map_err(okv_err)?;
        }

        tx.commit().map_err(okv_err)?;
        Ok(())
    }

    fn set_nx(&self, key: impl AsRef<[u8]>, val: impl AsRef<[u8]>) -> Result<bool> {
        let tx = self.db().begin_write().map_err(okv_err)?;

        {
            let mut table = tx.open_table(self.table()).map_err(okv_err)?;
            if table.get(key.as_ref()).map_err(okv_err)?.is_some() {
                return Ok(false);
            }
            table.insert(key.as_ref(), val.as_ref()).map_err(okv_err)?;
        }

        tx.commit().map_err(okv_err)?;
        Ok(true)
    }
}

impl DatabaseBackend for Redb {
    type Column = RedbColumn;
    fn create_or_open(
        env: okv_core::env::Env<Self>,
        db: &str,
    ) -> okv_core::error::Result<Self::Column> {
        let table = Self::Column {
            env,
            table: RedbTableInner::new(db.to_string(), |owner| {
                let table = TableDefinition::new(owner);
                BytesTable(table)
            }),
        };

        Ok(table)
    }
}

struct BytesTable<'a>(TableDefinition<'a, &'static [u8], &'static [u8]>);

self_cell!(
    pub(crate) struct RedbTableInner {
        owner: String,

        #[covariant]
        dependent: BytesTable,
    }
);
