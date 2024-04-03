use okv_core::{backend::DatabaseBackend, backend_async::DBColumnAsync};
use okv_core::{error::Result, traits::Innerable};
use worker::D1Type;

use super::okv_err;

pub struct CfD1 {
    env: worker::Env,
    binding: String,
}

pub struct CfD1Column {
    pub(crate) env: okv_core::env::Env<CfD1>,
    pub(crate) table: String,
}

impl CfD1 {
    pub fn new(env: worker::Env, binding: &str) -> Result<Self> {
        Ok(Self {
            env,
            binding: binding.to_string(),
        })
    }

    fn d1(&self) -> Result<worker::D1Database> {
        self.env.d1(&self.binding).map_err(okv_err)
    }
}

impl Innerable for CfD1 {
    type Inner = worker::Env;
    fn inner(&self) -> &Self::Inner {
        &self.env
    }
}

impl CfD1Column {
    fn d1(&self) -> Result<worker::D1Database> {
        self.env.db().d1()
    }

    fn str_key(&self, key: impl AsRef<[u8]>) -> Result<String> {
        String::from_utf8(key.as_ref().to_vec()).map_err(|e| {
            okv_core::error::Error::Unknown(
                "key is not valid utf8 - this is required for Cloudflare D1".to_string(),
            )
        })
    }
}

// for now, this is super annoying since a lot of the worker stuff isn't Send: https://github.com/cloudflare/workers-rs/issues/485
impl DBColumnAsync for CfD1Column {
    fn async_set(
        &self,
        key: impl AsRef<[u8]>,
        val: impl AsRef<[u8]> + Send,
    ) -> impl std::future::Future<Output = Result<()>> + Send {
        let key = self.str_key(key);

        #[inline]
        #[worker::send]
        async fn inner(env: &CfD1Column, key: Result<String>, val: impl AsRef<[u8]>) -> Result<()> {
            let d1 = env.d1()?;
            let val = D1Type::Blob(val.as_ref());
            let statement = d1.prepare("INSERT INTO ?1 (key, value) VALUES (?2, ?3)");

            let query = statement
                .bind(&[env.table.clone().into(), key?.into(), (&val).into()])
                .map_err(okv_err)?;

            match query.run().await {
                Err(e) => Err(okv_err(e)),
                Ok(res) => match res.error() {
                    Some(e) => Err(okv_err(e)),
                    None => Ok(()),
                },
            }
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
        async fn inner(env: &CfD1Column, key: Result<String>) -> Result<Option<Vec<u8>>> {
            let d1 = env.d1()?;
            let statement = d1.prepare("SELECT value FROM ?1 WHERE key = ?2");
            let query = statement
                .bind(&[env.table.clone().into(), key?.into()])
                .map_err(okv_err)?;
            query.first::<Vec<u8>>(None).await.map_err(okv_err)
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
            env: &CfD1Column,
            keys: Result<Vec<String>>,
        ) -> Result<Vec<Option<Vec<u8>>>> {
            todo!()
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
        async fn inner(env: &CfD1Column, key: Result<String>) -> Result<()> {
            todo!()
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
        async fn inner(env: &CfD1Column, key: Result<String>) -> Result<bool> {
            todo!()
        }

        inner(self, key)
    }
}

impl DatabaseBackend for CfD1 {
    type Column = CfD1Column;
    fn create_or_open(
        env: okv_core::env::Env<Self>,
        db: &str,
    ) -> okv_core::error::Result<Self::Column> {
        Ok(CfD1Column {
            env,
            table: db.to_string(),
        })
    }
}
