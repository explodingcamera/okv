<div>
  <img align="left" src="./okv.png" width="100px">
  <h1>OKV: Okay Key-Value Storage</h1>
</div>
 
[![docs.rs](https://img.shields.io/docsrs/okv?logo=rust)](https://docs.rs/okv) [![Crates.io](https://img.shields.io/crates/v/okv.svg?logo=rust)](https://crates.io/crates/okv) [![Crates.io](https://img.shields.io/crates/l/okv.svg)](./LICENSE-APACHE)

OKV is a versatile key-value storage library designed for Rust. It offers a simple yet powerful API, inspired by the [heed](https://github.com/meilisearch/heed) crate, and supports various databases and serialization formats. It doesn't give you the full power of a database, but it's a great choice for small to medium-sized projects that need a generic key-value storage solution that can be easily swapped out for a more powerful database later.

## Features

- **Multiple Database Backends**:
  - `memdb`: In-memory database for rapid prototyping and testing.
  - `rocksdb`: RocksDB integration for robust, disk-based storage.
    <!-- - `redb`: Rust-only embedded database. -->
    <!-- - `sqlite`: SQLite support for relational data storage. -->
- **Serialization Formats**:
  - `serde_json`: JSON serialization for human-readable data storage.
  - `rmp-serde`: MessagePack serialization for efficient binary format.

## Installation

Add OKV to your project:

```bash
cargo add okv
```

# Quick Start

```rust
use okv::{Env};
use okv::backend::memory::MemDB;

fn main() -> eyre::Result<()> {
    // initialize the storage backend
    let memdb = MemDB::new();
    let env = Env::new(memdb);

    // open a database with the specified key and value types
    let db = env.open::<&str, &str>("my_database")?;

    // Working with data
    db.set_nx("key", "val")?;
    assert_eq!(db.get("key")?, Some("val".to_string()));

    Ok(())
}
```

# Supported Types

OKV can work with any type that implements `serde::Serialize`/`serde::Deserialize`. Additionally, it supports the following types out of the box without any serialization overhead:

- Integer types: [`u8`], [`u16`], [`u32`], [`u64`], [`u128`], [`i8`], [`i16`], [`i32`], [`i64`], [`i128`]
- Basic types: `()`, [`&str`], [`String`], [`bool`]
- Binary data: u8 slices (`&[u8]`), byte vectors (`Vec<u8>`), and byte arrays (`[u8; N]`)

# Acknowledgements

Special thanks to the authors of the [heed](https://github.com/meilisearch/heed) crate for their inspiring work, which greatly influenced the API design and implementation of OKV.

# License

Licensed under either of [Apache License, Version 2.0](./LICENSE-APACHE) or [MIT license](./LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in OKV by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
