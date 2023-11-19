use std::borrow::Cow;

use super::{DatabaseBackend, DatabaseColumn};
use dashmap::{mapref::entry::OccupiedEntry, DashMap};

#[derive(Clone)]
pub struct MemDB<'c> {
    columns: DashMap<String, DashMap<Vec<u8>, Vec<u8>>>,
    marker: std::marker::PhantomData<&'c ()>,
}

pub struct MemDBColumn<'a> {
    _name: String,
    _env: &'a MemDB<'a>,
    entry: OccupiedEntry<'a, String, DashMap<Vec<u8>, Vec<u8>>>,
}

impl<'a> MemDBColumn<'a> {
    fn get<'c>(&self) -> &DashMap<Vec<u8>, Vec<u8>>
    where
        'a: 'c,
    {
        let c = self.entry.get();
        c
    }
}

impl<'a> MemDB<'a> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<'a> Default for MemDB<'a> {
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
{
    type Config = ();
    type Column = MemDBColumn<'c>;

    fn new(_connect_str: &str) -> Self {
        Self::default()
    }

    fn new_with_config(_config: Self::Config) -> Self {
        Self::default()
    }

    fn create_or_open(&'b self, name: &str) -> super::Result<Self::Column> {
        let entry = match self.columns.entry(name.to_string()) {
            dashmap::mapref::entry::Entry::Occupied(e) => e,
            dashmap::mapref::entry::Entry::Vacant(e) => e.insert_entry(DashMap::new()),
        };

        Ok(MemDBColumn {
            _name: name.to_owned(),
            _env: self,
            entry,
        })
    }
}

impl<'a, 'c> DatabaseColumn<'c> for MemDBColumn<'a> {
    fn set(&self, key: Cow<[u8]>, val: &[u8]) -> super::Result<()> {
        self.get().insert(key.to_vec(), val.to_vec());
        Ok(())
    }

    fn get(&self, key: Cow<[u8]>) -> Option<Vec<u8>> {
        let db = self.get();
        let val = db.get(&key.to_vec())?;
        Some(val.clone())
    }
}
