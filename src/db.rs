use std::cell::OnceCell;
use std::marker::PhantomData;

use crate::backend::{DatabaseBackend, DatabaseBackendRef};
use crate::env::Env;
use crate::traits::{BytesDecode, BytesDecodeOwned, BytesEncode};
use crate::Result;

pub struct OKV<'e, K, V, D>
where
    D: DatabaseBackend,
{
    name: String,
    env: &'e Env<D>,
    _phantom: PhantomData<(K, V)>,
}

pub(crate) fn new<'e, K, V, D>(env: &'e Env<D>, name: &str) -> OKV<'e, K, V, D>
where
    D: DatabaseBackend,
{
    OKV {
        name: name.to_string(),
        env,
        _phantom: PhantomData,
    }
}

impl<Key, Val, D> OKV<'_, Key, Val, D>
where
    D: DatabaseBackend,
{
    pub fn set<'a, 'b>(&'a mut self, key: &'b Key::EItem, val: &'b Val::EItem) -> Result<()>
    where
        Key: BytesEncode<'b>,
        Val: BytesEncode<'b>,
    {
        let key_bytes = Key::bytes_encode(key)?;
        let val_bytes = Val::bytes_encode(val)?;
        self.env.db().set(&self.name, &key_bytes, &val_bytes)?;
        Ok(())
    }

    pub fn get<'k, 'v>(&self, key: &'k Key::EItem) -> Result<Val::DItem>
    where
        Key: BytesEncode<'k>,
        Val: BytesDecodeOwned,
    {
        let key_bytes = Key::bytes_encode(key)?;

        let val_bytes = self
            .env
            .db()
            .get(&self.name, &key_bytes.to_vec())
            .ok_or_else(|| crate::Error::KeyNotFound {
                key: key_bytes.to_vec(),
            })?;

        let res = Val::bytes_decode_owned(&val_bytes)?;
        Ok(res)
    }
}

impl<Key, Val, D> OKV<'_, Key, Val, D>
where
    D: DatabaseBackendRef,
{
    pub fn get_ref<'a, 'b>(&'a self, key: &'b Key::EItem) -> Result<Val::DItem>
    where
        Key: BytesEncode<'b>,
        Val: BytesDecode<'a>,
    {
        let key_bytes = Key::bytes_encode(key)?;
        let val_bytes = self
            .env
            .db()
            .get_ref(&self.name, &key_bytes.to_vec())
            .ok_or_else(|| crate::Error::KeyNotFound {
                key: key_bytes.to_vec(),
            })?;

        let deserialized = Val::bytes_decode(&val_bytes)?;
        Ok(deserialized)
    }
}

pub struct Res<'a, Val>
where
    Val: BytesDecode<'a>,
{
    val: Vec<u8>,
    deserialized: OnceCell<Val::DItem>,
}

impl<'a, Val> Res<'a, Val>
where
    Val: BytesDecode<'a>,
{
    fn new(val: Vec<u8>) -> Self {
        Self {
            val,
            deserialized: OnceCell::new(),
        }
    }

    pub fn bytes(&self) -> &[u8] {
        &self.val
    }

    pub fn deserialize(&'a self) -> Result<Val::DItem> {
        let deserialized = Val::bytes_decode(&self.val)?;
        Ok(deserialized)
    }
}
