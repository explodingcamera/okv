use std::sync::Arc;

use crate::{
    backend::DatabaseBackend,
    db::{self, Database},
    Result,
};

/// A database environment
/// Can be cloned.
pub struct Env<'a, D: DatabaseBackend<'a>>(Arc<EnvInner<'a, D>>);

impl<'a, D> Clone for Env<'a, D>
where
    D: DatabaseBackend<'a>,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<'a, D: DatabaseBackend<'a>> Env<'a, D> {
    pub(crate) fn db(&self) -> &D {
        &self.0.db
    }

    /// Returns a reference to the underlying column.
    /// Can be used to access the database directly.
    pub fn inner(&self) -> &D::Inner {
        self.0.db.inner()
    }

    /// Create a new environment backed by the given database.
    pub fn new(db: D) -> Self {
        Self(Arc::new(EnvInner {
            db,
            marker: Default::default(),
        }))
    }

    /// Open or create a database.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the database.
    ///
    /// # Generic Arguments
    ///
    /// * `V` - The value type. This must implement [`crate::BytesEncode`].
    /// * `B` - The value type. This must implement [`crate::BytesEncode`], [`crate::BytesDecode`] and [`crate::BytesDecodeOwned`].
    ///
    /// # Examples
    ///
    /// ```
    /// use okv::{Env, mem::MemDB};
    /// let backend = MemDB::new();
    /// let env = Env::new(backend);
    /// let mut db = env.open::<&str, &str>("test").unwrap();
    /// ```
    pub fn open<K, V>(&'a self, name: &str) -> Result<Database<'a, K, V, D>> {
        db::new(self, name)
    }
}

struct EnvInner<'a, D: DatabaseBackend<'a>> {
    pub(crate) db: D,
    marker: std::marker::PhantomData<&'a ()>,
}
