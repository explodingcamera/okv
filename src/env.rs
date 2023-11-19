use std::sync::Arc;

use crate::{
    backend::DatabaseBackend,
    db::{self, Database},
    Result,
};

// 'c: Columns
// 'd: Database backend
pub struct Env<'d, 'c, D>(Arc<EnvInner<'d, 'c, D>>)
where
    D: DatabaseBackend<'d, 'c>;

impl<'b, 'c, D> Clone for Env<'b, 'c, D>
where
    D: DatabaseBackend<'b, 'c>,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<'b, 'c, D> Env<'b, 'c, D>
where
    D: DatabaseBackend<'b, 'c>,
    'b: 'c,
{
    pub(crate) fn db(&self) -> &D {
        &self.0.db
    }

    pub fn new(db: D) -> Self {
        Self(Arc::new(EnvInner {
            db,
            marker: std::marker::PhantomData,
        }))
    }

    pub fn open<K, V>(&'b self, name: &str) -> Result<Database<K, V, D>> {
        db::new(self, name)
    }
}

struct EnvInner<'d, 'c, D: DatabaseBackend<'d, 'c>> {
    pub(crate) db: D,
    marker: std::marker::PhantomData<(&'d (), &'c ())>,
}
