use std::sync::Arc;

use crate::{
    backend::DatabaseBackend,
    db::{self, OKV},
    Result,
};

#[derive(Clone)]
pub struct Env<D>(Arc<EnvInner<D>>)
where
    D: DatabaseBackend;

impl<D> Env<D>
where
    D: DatabaseBackend,
{
    pub(crate) fn db(&self) -> &D {
        &self.0.db
    }

    pub fn new(db: D) -> Self {
        Self(Arc::new(EnvInner { db }))
    }

    pub fn open<K, V>(&self, name: &str) -> Result<OKV<K, V, D>> {
        self.0.db.create_or_open(name)?;
        Ok(db::new(self, name))
    }
}

struct EnvInner<D: DatabaseBackend> {
    pub(crate) db: D,
}
