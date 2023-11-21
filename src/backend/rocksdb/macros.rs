use crate::backend::DatabaseColumn;
use crate::backend::{DatabaseBackend, DatabaseColumn, DatabaseColumnRef};

#[macro_export]
macro_rules! implement_column {
    ($name:ident) => {
        impl<'a> DatabaseColumn for $name<'a> {
            fn set(&self, key: impl AsRef<[u8]>, val: &[u8]) -> Result<()> {
                self._env.db.put_cf(&self.cf_handle, key, val)?;
                Ok(())
            }

            fn get(&self, key: impl AsRef<[u8]>) -> Result<Option<Vec<u8>>> {
                match self._env.db.get_cf(&self.cf_handle, key)? {
                    Some(x) => Ok(Some(x.to_vec())),
                    None => Ok(None),
                }
            }

            fn contains(&self, key: impl AsRef<[u8]>) -> Result<bool> {
                match self._env.db.get_cf(&self.cf_handle, key)? {
                    Some(_) => Ok(true),
                    None => Ok(false),
                }
            }

            fn delete(&self, key: impl AsRef<[u8]>) -> Result<()> {
                self._env.db.delete_cf(&self.cf_handle, key)?;
                Ok(())
            }

            fn clear(&self) -> Result<()> {
                self._env.db.drop_cf(&self._name)?;
                self._env
                    .db
                    .create_cf(&self._name, &rocksdb::Options::default())?;

                Ok(())
            }

            fn get_multi<I>(&self, keys: I) -> Result<Vec<Option<Vec<u8>>>>
            where
                I: IntoIterator,
                I::Item: AsRef<[u8]>,
            {
                let keys = keys.into_iter().map(|key| (&self.cf_handle, key));
                let values = self._env.db.multi_get_cf(keys);
                let values = values
                    .into_iter()
                    .collect::<std::result::Result<Vec<_>, _>>()?;
                Ok(values)
            }
        }
    };
}

#[macro_export]
macro_rules! implement_backend {
    ($name:ident, $col:ident, $db:ident) => {
        impl<'a> DatabaseBackend<'a> for $name<'a> {
            type Column = $col<'a>;
            fn create_or_open(&'a self, name: &str) -> super::Result<Self::Column> {
                if let Some(handle) = self.db.cf_handle(name) {
                    return Ok($col {
                        _name: name.to_owned(),
                        _env: self,
                        cf_handle: handle,
                    });
                };

                let cf_opts = rocksdb::Options::default();
                self.db.create_cf(name, &cf_opts)?;
                let handle = self.db.cf_handle(name).ok_or(Error::DatabaseNotFound {
                    db: name.to_string(),
                })?;

                Ok($col {
                    _name: name.to_owned(),
                    _env: self,
                    cf_handle: handle,
                })
            }
        }

        impl<'a> crate::backend::Innerable for $name<'a> {
            type Inner = $db;
            fn inner(&self) -> &Self::Inner {
                &self.db
            }
        }
    };
}
