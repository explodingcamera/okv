mod kv;
pub use kv::{CfKV, CfKVColumn};
use okv_core::error::Error;

pub(crate) fn okv_err(e: impl Into<worker::Error>) -> Error {
    Error::DatabaseBackend(Box::new(e.into()))
}

#[cfg(feature = "d1")]
mod d1;
#[cfg(feature = "d1")]
pub use d1::{CfD1, CfD1Column};
