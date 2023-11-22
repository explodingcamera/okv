use crate::{backend::DatabaseBackend, db::Database, Result};
use std::sync::Arc;

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
        Database::new(self, name)
    }

    /// Same as [`Env::open`] but you can specify the type of the key and value using a tuple.
    /// This is useful when you want to reuse the same type for multiple databases.
    pub fn open_tupel<T: DatabaseType>(
        &'a self,
        name: &str,
    ) -> Result<Database<'a, T::Key, T::Val, D>> {
        Database::new(self, name)
    }

    #[cfg(feature = "unstable_lasydb")]
    /// Open or create a database lazily.
    /// This is useful for sharing the same database across threads.
    /// Alternatively, you can use [`Env::clone`] to share environments across threads and use [`Env::open`] to open the database.
    pub fn open_lazy<K, V>(&'a self, name: &str) -> crate::db::DatabaseLazy<'a, K, V, D> {
        crate::db::DatabaseLazy::new(self, name)
    }
}

pub trait DatabaseType {
    type Key;
    type Val;
}

impl<'k, 'v, K, V> DatabaseType for (K, V)
where
    K: crate::traits::BytesEncode<'k>,
    V: crate::traits::BytesEncode<'v>,
{
    type Key = K;
    type Val = V;
}

struct EnvInner<'a, D: DatabaseBackend<'a>> {
    pub(crate) db: D,
    marker: std::marker::PhantomData<&'a ()>,
}
