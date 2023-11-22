use std::{marker::PhantomData, sync::Arc};

use crate::{backend::DatabaseBackend, Database, Env, Result};

#[derive(Clone)]
pub struct DatabaseLazy<'a, K, V, D: DatabaseBackend<'a>> {
    pub env: Arc<&'a Env<'a, D>>,
    pub name: String,
    pub _phantom: PhantomData<(K, V)>,
}

impl<'a, K, V, D: DatabaseBackend<'a>> DatabaseLazy<'a, K, V, D> {
    pub(crate) fn new(env: &'a Env<'a, D>, name: &str) -> Self {
        Self {
            env: Arc::new(env),
            name: name.to_string(),
            _phantom: PhantomData,
        }
    }

    pub fn open(&self) -> Result<Database<'a, K, V, D>> {
        Database::new(&self.env, &self.name)
    }
}
