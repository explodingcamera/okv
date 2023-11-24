//! OKV - Okay Key-Value Storage
#![doc(test(
    no_crate_inject,
    attr(
        deny(warnings, rust_2018_idioms),
        allow(dead_code, unused_assignments, unused_variables)
    )
))]
#![warn(
    missing_docs,
    missing_debug_implementations,
    rust_2018_idioms,
    unreachable_pub
)]

#[doc(inline)]
pub use okv_core::db::Database;

#[doc(inline)]
pub use okv_core::db::DatabaseTransaction;

#[doc(inline)]
pub use okv_core::env::Env;

#[doc(inline)]
pub use okv_core::error::{DecodeError, EncodeError, Error};

#[doc(inline)]
/// Serialization types
pub use okv_core::types;

pub mod backend {
    //! Database backends
    //!
    //! * [`memory`] - In-memory database backend (requires `memory` feature)
    //! * [`rocksdb`] - RocksDB database backend (requires `rocksdb` feature). Based on <https://crates.io/crates/rocksdb>.
    // //! * [`sqlite`] - Sqlite database backend (requires `sqlite` feature). Based on <https://crates.io/crates/rusqlite>.

    #[cfg(feature = "rocksdb")]
    #[doc(inline)]
    pub use okv_rocksdb as rocksdb;

    #[cfg(feature = "memory")]
    #[doc(inline)]
    pub use okv_memory as memory;

    // TODO
    // #[cfg(feature = "sqlite")]
    // #[doc(inline)]
    // pub use okv_sqlite as sqlite;
}

#[doc(inline)]
pub use okv_core::traits::{
    DBCommon, DBCommonClear, DBCommonDelete, DBCommonIter, DBCommonIterPrefix, DBCommonRef,
    DBCommonRefBatch,
};
