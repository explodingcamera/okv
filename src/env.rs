use std::sync::Arc;

use crate::{
    backend::DatabaseBackend,
    db::{self, Database},
    Result,
};

/// A database environment
/// Can be cloned.
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

    /// Returns a reference to the underlying column.
    /// Can be used to access the database directly.
    pub fn inner(&self) -> &D::Inner {
        self.0.db.inner()
    }

    /// Create a new environment backed by the given database.
    pub fn new(db: D) -> Self {
        Self(Arc::new(EnvInner {
            db,
            marker: std::marker::PhantomData,
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
    /// let backend = MemDB::new();
    /// let env = Env::new(backend);
    /// let mut db = env.open::<&str, SerdeJson<Test>>("test")?;
    /// ```
    pub fn open<K, V>(&'b self, name: &str) -> Result<Database<K, V, D>> {
        db::new(self, name)
    }
}

struct EnvInner<'d, 'c, D: DatabaseBackend<'d, 'c>> {
    pub(crate) db: D,
    marker: std::marker::PhantomData<(&'d (), &'c ())>,
}
