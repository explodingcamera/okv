mod kv;

use okv_core::error::Error;

pub(crate) fn okv_err(e: impl Into<worker::Error>) -> Error {
    Error::DatabaseBackend(Box::new(e.into()))
}

pub use kv::{CfKV, CfKVColumn};
