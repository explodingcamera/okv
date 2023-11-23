use crate::{Innerable, Result};

// #[cfg(feature = "unstable_any")]
/// Any database backend (requires `unstable_any` feature)
// pub mod any;

#[cfg(feature = "memdb")]
/// In-memory database backend (requires `memdb` feature)
pub mod mem;

#[cfg(feature = "rocksdb")]
/// RocksDB database backend (requires `rocksdb` feature)
pub mod rocksdb;

mod traits;
pub(crate) use traits::*;
