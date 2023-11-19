#![doc = include_str!("../README.md")]

mod error;
mod traits;
pub use traits::*;
pub mod types;

mod backend;
mod db;
mod env;

pub use error::*;

#[cfg(feature = "memdb")]
pub use backend::mem;

#[cfg(feature = "rocksdb")]
pub use backend::rocksdb;

pub use db::{Database, RefValue};
pub use env::Env;

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
struct Test {
    name: String,
    age: u32,
}

#[cfg(test)]
mod test {
    use std::thread;

    use crate::backend::mem::MemDB;
    use crate::env::Env;
    use crate::types::serde::SerdeJson;
    use crate::Test;

    #[test]
    fn rocksdb() -> crate::Result<()> {
        let test = Test {
            name: "hello".to_string(),
            age: 10,
        };

        use crate::backend::rocksdb::RocksDb;
        let backend = RocksDb::new("database/rocks")?;

        let env = Env::new(backend);
        let mut db = env.open::<&str, SerdeJson<Test>>("test")?;
        db.set("hello", &test)?;
        let res = db.get("hello")?;
        assert_eq!(res, test);

        let env2 = env.clone();
        let handler = thread::spawn(move || {
            let test = Test {
                name: "hello".to_string(),
                age: 10,
            };
            let db = env2.open::<&str, SerdeJson<Test>>("test").unwrap();
            let mut db2 = db.clone();
            db2.set("hello", &test).unwrap();
            let res = db.get("hello").unwrap();
            assert_eq!(res, test);
        });

        handler.join().unwrap();
        let db = env.open::<&str, SerdeJson<Test>>("test").unwrap();
        let _res = db.get("hello")?;
        Ok(())
    }

    #[test]
    fn test() -> crate::Result<()> {
        let test = Test {
            name: "hello".to_string(),
            age: 10,
        };
        dbg!("test");

        let backend = MemDB::new();
        let env = Env::new(backend);
        let mut db = env.open::<&str, SerdeJson<Test>>("test")?;
        db.set("hello", &test)?;
        let res = db.get("hello")?;
        assert_eq!(res, test);

        let env2 = env.clone();
        let handler = thread::spawn(move || {
            let test = Test {
                name: "hello".to_string(),
                age: 10,
            };
            let db = env2.open::<&str, SerdeJson<Test>>("test").unwrap();
            let mut db2 = db.clone();
            db2.set("hello", &test).unwrap();
            let res = db.get("hello").unwrap();
            assert_eq!(res, test);
        });

        handler.join().unwrap();
        let db = env.open::<&str, SerdeJson<Test>>("test").unwrap();
        let _res = db.get("hello")?;
        Ok(())
    }
}
