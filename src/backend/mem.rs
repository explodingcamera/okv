use super::DatabaseBackend;
use dashmap::DashMap;
use std::{cell::RefCell, collections::HashMap};

#[derive(Clone)]
pub struct MemDBShared {
    dbs: DashMap<String, HashMap<Vec<u8>, Vec<u8>>>,
}

pub struct MemDB {
    dbs: RefCell<HashMap<String, HashMap<Vec<u8>, Vec<u8>>>>,
}

impl MemDBShared {
    pub fn new() -> Self {
        Self {
            dbs: DashMap::new(),
        }
    }
}

impl MemDB {
    pub fn new() -> Self {
        Self {
            dbs: RefCell::new(HashMap::new()),
        }
    }
}

impl DatabaseBackend for MemDBShared {
    fn create_or_open(&self, db: &str) -> crate::Result<()> {
        self.dbs.insert(db.to_string(), HashMap::new());
        Ok(())
    }

    fn set(&self, db: &str, key: &[u8], val: &[u8]) -> super::Result<()> {
        let mut db = self
            .dbs
            .get_mut(db)
            .ok_or_else(|| crate::Error::DatabaseNotFound { db: db.to_string() })?;
        db.insert(key.to_vec(), val.to_vec());
        Ok(())
    }

    fn get(&self, db: &str, key: &[u8]) -> Option<Vec<u8>> {
        let db = self.dbs.get(db)?;
        let val = db.get(key)?;
        Some(val.clone())
    }
}

impl DatabaseBackend for MemDB {
    fn create_or_open(&self, db: &str) -> crate::Result<()> {
        self.dbs.borrow_mut().insert(db.to_string(), HashMap::new());
        Ok(())
    }

    fn set(&self, name: &str, key: &[u8], val: &[u8]) -> super::Result<()> {
        let mut db = self.dbs.borrow_mut();
        let db = db
            .get_mut(name)
            .ok_or_else(|| crate::Error::DatabaseNotFound {
                db: name.to_string(),
            })?;

        db.insert(key.to_vec(), val.to_vec())
            .ok_or_else(|| crate::Error::DatabaseNotFound {
                db: name.to_string(),
            })?;

        Ok(())
    }

    fn get(&self, name: &str, key: &[u8]) -> Option<Vec<u8>> {
        let db = self.dbs.borrow();
        let db = db.get(name)?;
        let val = db.get(key)?;
        Some(val.clone())
    }
}
