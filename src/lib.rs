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

pub use okv_core::db::Database;
pub use okv_core::env::Env;

pub mod backend {
    //! Database Backends
    //!
    //! * [`memory`] - In-memory database backend (requires `memory` feature)
    //! * [`rocksdb`] - RocksDB database backend (requires `rocksdb` feature). Based on the `rocksdb` crate.
    //! * [`sqlite`] - Sqlite database backend (requires `sqlite` feature). Based on the `rusqlite` crate.

    #[cfg(feature = "rocksdb")]
    #[doc(inline)]
    pub use okv_rocksdb as rocksdb;

    // TODO
    #[cfg(feature = "memory")]
    #[doc(inline)]
    pub use okv_memory as memory;

    // TODO
    #[cfg(feature = "sqlite")]
    #[doc(inline)]
    pub use okv_sqlite as sqlite;
}
