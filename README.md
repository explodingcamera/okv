# OKV - Okay Key-Value Storage

## Credits

A lot of the inspiration for the okv api comes from the amazing [heed](https://github.com/meilisearch/heed) crate. Thanks to the authors of heed for their work!

## Features

`default`: `memdb` and `serde-json`

### Databases
`memdb`: In-memory database
`rocksdb`: RocksDB database
`sqlite`: SQLite database

### Serializers
`serde-json`: JSON serialization
`serde-rmp`: MessagePack serialization

## Usage

```rust
use okv::{Database, MemDB, SerdeJson};

struct Test {
    name: String,
    age: u32,
}

fn main() -> Result<()> {
    let mut storage = MemDB::new(); // or RocksDB::new("path/to/db") or SQLite::new("path/to/db")
    let env = Env::new(storage);

    let mut db = env.open::<&str, SerdeJson<Test>>("my_database")?; // or SerdeRmp

    let test = Test {
        name: "John".to_string(),
        age: 32,
    };

    db.set("test", &test)?;
    let result = db.get("test")?;

    assert_eq!(result, Some(test));
}
```

### Supported types

Any type that implements `Serialize` can be used as a key or value. The following types are supported out of the box:

- `u8`, `u16`, `u32`, `u64`, `u128`, `usize`
- `i8`, `i16`, `i32`, `i64`, `i128`, `isize`
- `()`
- `&str`, `String`
- `&[u8]`, `Vec<u8>`