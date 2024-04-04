use futures::future::join_all;
use okv_core::backend_async::sync_fallback;
use okv_core::{backend::DatabaseBackend, backend_async::DBColumnAsync};
use okv_core::{error::Result, traits::Innerable};

use super::okv_err;
sync_fallback!(CfKVColumn);

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
            "{}-{}",
            self.prefix,
            String::from_utf8(key.as_ref().to_vec()).map_err(|_| {
                okv_core::error::Error::Unknown(
                    "key is not valid utf8 - this is required for Cloudflare KV".to_string(),
                )
            })?
        ))
    }
}

// for now, this is super annoying since a lot of the worker stuff isn't Send: https://github.com/cloudflare/workers-rs/issues/485
impl DBColumnAsync for CfKVColumn {
    fn async_set(
        &self,
        key: impl AsRef<[u8]>,
        val: impl AsRef<[u8]> + Send,
    ) -> impl std::future::Future<Output = Result<()>> + Send {
        let key = self.str_key(key);

        #[inline]
        #[worker::send]
        async fn inner(env: &CfKVColumn, key: Result<String>, val: impl AsRef<[u8]>) -> Result<()> {
            env.kv()?
                .put_bytes(&key?, val.as_ref())
                .map_err(okv_err)?
                .execute()
                .await
                .map_err(okv_err)
        }

        inner(self, key, val)
    }

    fn async_get(
        &self,
        key: impl AsRef<[u8]>,
    ) -> impl std::future::Future<Output = Result<Option<Vec<u8>>>> + Send {
        let key = self.str_key(key);

        #[inline]
        #[worker::send]
        async fn inner(env: &CfKVColumn, key: Result<String>) -> Result<Option<Vec<u8>>> {
            env.kv()?.get(&key?).bytes().await.map_err(okv_err)
        }

        inner(self, key)
    }

    fn async_get_multi<I>(
        &self,
        keys: I,
    ) -> impl std::future::Future<Output = Result<Vec<Option<Vec<u8>>>>> + Send
    where
        I: IntoIterator,
        I::Item: AsRef<[u8]>,
    {
        let keys = keys
            .into_iter()
            .map(|k| self.str_key(k))
            .collect::<Result<Vec<_>>>();

        #[inline]
        #[worker::send]
        async fn inner(
            env: &CfKVColumn,
            keys: Result<Vec<String>>,
        ) -> Result<Vec<Option<Vec<u8>>>> {
            let keys = keys?;
            let futures = keys
                .into_iter()
                .map(|key| async move { env.kv()?.get(&key).bytes().await.map_err(okv_err) })
                .collect::<Vec<_>>(); // Collecting futures here

            join_all(futures).await.into_iter().collect() // Awaiting all futures concurrently
        }

        inner(self, keys)
    }

    fn async_delete(
        &self,
        key: impl AsRef<[u8]>,
    ) -> impl std::future::Future<Output = Result<()>> + Send {
        let key = self.str_key(key);

        #[inline]
        #[worker::send]
        async fn inner(env: &CfKVColumn, key: Result<String>) -> Result<()> {
            env.kv()?.delete(&key?).await.map_err(okv_err)
        }

        inner(self, key)
    }

    fn async_contains(
        &self,
        key: impl AsRef<[u8]>,
    ) -> impl std::future::Future<Output = Result<bool>> + Send {
        let key = self.str_key(key);

        #[inline]
        #[worker::send]
        async fn inner(env: &CfKVColumn, key: Result<String>) -> Result<bool> {
            env.kv()?
                .get(&key?)
                .bytes()
                .await
                .map_err(okv_err)
                .map(|res| res.is_some())
        }

        inner(self, key)
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
