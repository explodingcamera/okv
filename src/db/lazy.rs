use std::marker::PhantomData;

use crate::{backend::DatabaseBackend, Database, Env, Result};

#[derive(Clone)]
pub struct DatabaseLazy<K, V, D: DatabaseBackend> {
    pub env: Env<D>,
    pub name: String,
    pub _phantom: PhantomData<(K, V)>,
}

impl<K, V, D: DatabaseBackend> DatabaseLazy<K, V, D> {
    pub(crate) fn new(env: Env<D>, name: &str) -> Self {
        Self {
            env,
            name: name.to_string(),
            _phantom: PhantomData,
        }
    }

    pub fn open(&self) -> Result<Database<K, V, D>> {
        Database::new(&self.env, &self.name)
    }
}
