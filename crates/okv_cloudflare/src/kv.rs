use okv_core::{
    backend::DatabaseBackend, backend_async::DBColumnAsync, error::Result, traits::Innerable,
};

use crate::okv_err;

pub struct CfKV {
    env: worker::Env,
    namespace: String,
}

pub struct CfKVColumn {
    pub(crate) env: okv_core::env::Env<CfKV>,
    pub(crate) prefix: String,
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

impl DBColumnAsync for CfKVColumn {
    fn async_set(
        &self,
        key: impl AsRef<[u8]>,
        val: impl AsRef<[u8]>,
    ) -> impl std::future::Future<Output = Result<()>> + Send {
        async { todo!() }
    }

    fn async_get(
        &self,
        key: impl AsRef<[u8]>,
    ) -> impl std::future::Future<Output = Result<Option<Vec<u8>>>> + Send {
        async { todo!() }
    }

    fn async_get_multi<I>(
        &self,
        keys: I,
    ) -> impl std::future::Future<Output = Result<Vec<Option<Vec<u8>>>>> + Send
    where
        I: IntoIterator,
        I::Item: AsRef<[u8]>,
    {
        async { todo!() }
    }

    fn async_delete(
        &self,
        key: impl AsRef<[u8]>,
    ) -> impl std::future::Future<Output = Result<()>> + Send {
        async { todo!() }
    }

    fn async_contains(
        &self,
        key: impl AsRef<[u8]>,
    ) -> impl std::future::Future<Output = Result<bool>> + Send {
        async { todo!() }
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
