# OKV - Okay Key-Value Storage

[![docs.rs](https://img.shields.io/docsrs/okv?logo=rust)](https://docs.rs/okv) [![Crates.io](https://img.shields.io/crates/v/okv.svg?logo=rust)](https://crates.io/crates/okv) [![Crates.io](https://img.shields.io/crates/l/okv.svg)](./LICENSE-APACHE) 

OKV is a versatile key-value storage library designed for Rust. It offers a simple yet powerful API, inspired by the [heed](https://github.com/meilisearch/heed) crate, and supports various databases and serialization formats.

## Features

- **Multiple Database Backends**: 
  - `memdb`: In-memory database for rapid prototyping and testing.
  - `rocksdb`: RocksDB integration for robust, disk-based storage.
  <!-- - `sqlite`: SQLite support for relational data storage. -->
- **Serialization Formats**: 
  - `serde-json`: JSON serialization for human-readable data storage.
  - `serde-rmp`: MessagePack serialization for efficient binary format.

## Installation

Add OKV to your project:

```bash
cargo add okv
```

# Quick Start

```rust
use okv::{Database, Env, Result};
use okv::mem::MemDB;
use okv::types::serde::SerdeJson;

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
struct Test {
    name: String,
    age: u32,
}

fn main() -> Result<()> {
    // Initialize storage backend
    let mut storage = MemDB::new();
    let env = Env::new(storage);

    // Open a database with specified serializers
    let mut db = env.open::<&str, SerdeJson<Test>>("my_database")?;
//  let mut db = env.open::<&str, SerdeRmp<Test>>("my_database")?;
//  let mut db = env.open::<&str, &[u8]>("my_database")?;

    // Working with data
    let test = Test { name: "John".to_string(), age: 32 };
    db.set("test", &test)?;
    let result = db.get("test")?;

    // Verify the operation
    assert_eq!(result, Some(test));

    Ok(())
}
```

# Supported Types

OKV can work with any type that implements Serialize. Additionally, it supports the following types out of the box without any serialization overhead:

* Integer types: [`u8`], [`u16`], [`u32`], [`u64`], [`u128`], [`i8`], [`i16`], [`i32`], [`i64`], [`i128`]
* Basic types: `()`, [`&str`], [`String`], [`bool`]
* Binary data: u8 slices (`&[u8]`), byte vectors (`Vec<u8>`), and byte arrays (`[u8; N]`)

# Acknowledgements

Special thanks to the authors of the [heed](https://github.com/meilisearch/heed) crate for their inspiring work, which greatly influenced the development of OKV.

# License

Licensed under either of [Apache License, Version 2.0](./LICENSE-APACHE) or [MIT license](./LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in OKV by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.