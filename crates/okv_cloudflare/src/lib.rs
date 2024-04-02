use okv_core::{
    backend::{DBColumn, DatabaseBackend},
    error::{Error, Result},
    traits::Innerable,
};
use worker::kv::ToRawKvValue;

pub(crate) fn okv_err(e: impl Into<worker::Error>) -> Error {
    Error::DatabaseBackend(Box::new(e.into()))
}

pub struct CfKV {
    env: worker::Env,
    namespace: String,
}

impl CfKV {
    pub fn new(env: worker::Env, namespace: &str) -> Result<Self> {
        Ok(Self {
            env,
            namespace: namespace.to_string(),
        })
    }

    fn kv(&self) -> Result<worker::kv::KvStore> {
        self.env.kv(&self.namespace).map_err(okv_err)
    }
}

impl Innerable for CfKV {
    type Inner = worker::Env;
    fn inner(&self) -> &Self::Inner {
        &self.env
    }
}

pub struct CfKVColumn {
    pub(crate) env: okv_core::env::Env<CfKV>,
    pub(crate) prefix: String,
}

impl CfKVColumn {
    fn kv(&self) -> Result<worker::kv::KvStore> {
        self.env.db().kv()
    }

    fn str_key(&self, key: impl AsRef<[u8]>) -> Result<String> {
        Ok(format!(
            "{}{}",
            self.prefix,
            String::from_utf8(key.as_ref().to_vec()).map_err(|e| {
                okv_core::error::Error::Unknown("key is not valid utf8".to_string())
            })?
        ))
    }
}

impl DBColumn for CfKVColumn {
    fn contains(&self, key: impl AsRef<[u8]>) -> okv_core::error::Result<bool> {
        let key = &self.str_key(key)?;
        self.kv()?
            .get(key)
            .bytes()
            .map_err(okv_err)
            .map(|v| v.is_some());

        Ok(true)
    }

    fn delete(&self, key: impl AsRef<[u8]>) -> Result<()> {
        let tx = self.db().begin_write().map_err(okv_err)?;

        {
            let mut table = tx.open_table(self.table()).map_err(okv_err)?;
            table.remove(key.as_ref()).map_err(okv_err)?;
        }

        tx.commit().map_err(okv_err)?;
        Ok(())
    }

    fn get(&self, key: impl AsRef<[u8]>) -> Result<Option<Vec<u8>>> {
        let tx = self.db().begin_read().map_err(okv_err)?;
        let table = tx.open_table(self.table()).map_err(okv_err)?;
        let res = table.get(key.as_ref()).map_err(okv_err)?;
        Ok(res.map(|v| v.value().to_vec()))
    }

    fn get_multi<I>(&self, keys: I) -> Result<Vec<Option<Vec<u8>>>>
    where
        I: IntoIterator,
        I::Item: AsRef<[u8]>,
    {
        let mut res = Vec::new();
        let tx = self.db().begin_read().map_err(okv_err)?;
        let table = tx.open_table(self.table()).map_err(okv_err)?;
        for key in keys {
            let val = table.get(key.as_ref()).map_err(okv_err)?;
            res.push(val.map(|v| v.value().to_vec()));
        }

        Ok(res)
    }

    fn set(&self, key: impl AsRef<[u8]>, val: impl AsRef<[u8]>) -> Result<()> {
        let tx = self.db().begin_write().map_err(okv_err)?;

        {
            let mut table = tx.open_table(self.table()).map_err(okv_err)?;
            table.insert(key.as_ref(), val.as_ref()).map_err(okv_err)?;
        }

        tx.commit().map_err(okv_err)?;
        Ok(())
    }

    fn set_nx(&self, key: impl AsRef<[u8]>, val: impl AsRef<[u8]>) -> Result<bool> {
        let tx = self.db().begin_write().map_err(okv_err)?;

        {
            let mut table = tx.open_table(self.table()).map_err(okv_err)?;
            if table.get(key.as_ref()).map_err(okv_err)?.is_some() {
                return Ok(false);
            }
            table.insert(key.as_ref(), val.as_ref()).map_err(okv_err)?;
        }

        tx.commit().map_err(okv_err)?;
        Ok(true)
    }
}

impl DatabaseBackend for CfKV {
    type Column = CfKVColumn;
    fn create_or_open(
        env: okv_core::env::Env<Self>,
        db: &str,
    ) -> okv_core::error::Result<Self::Column> {
        Ok(CfKVColumn {
            env,
            prefix: db.to_string(),
        })
    }
}
