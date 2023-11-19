# OKV - Okay Key-Value Storage

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
use okv::{Database, MemDB, SerdeJson, Env, Result};

struct Test {
    name: String,
    age: u32,
}

fn main() -> Result<()> {
    // Initialize storage backend
    let mut storage = MemDB::new();
    let env = Env::new(storage);

    // Open a database with specified serializers
    let mut db = env.open::<&str, SerdeJson<Test>>("my_database")?; // or SerdeRmp

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

* Integer types: u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize
* Basic types: (), &str, String
* Binary data: &[u8], Vec<u8>

# Acknowledgements

Special thanks to the authors of the [heed](https://github.com/meilisearch/heed) crate for their inspiring work, which greatly influenced the development of OKV.

# License

Licensed under either of [Apache License, Version 2.0](./LICENSE-APACHE) or [MIT license](./LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in OKV by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.