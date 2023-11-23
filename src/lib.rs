#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

mod db;
mod env;
mod error;
mod traits;
/// Types for serialization
pub mod types;
pub use db::Database;
pub use env::Env;
pub use error::*;
pub use traits::*;

mod backend;
#[cfg(feature = "memdb")]
pub use backend::mem;

#[cfg(feature = "rocksdb")]
pub use backend::rocksdb;

#[cfg(test)]
mod test {
    use std::thread;

    use crate::backend::mem::MemDB;
    use crate::types::serde::SerdeJson;
    use crate::Env;

    #[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
    struct Test {
        name: String,
        age: u32,
    }
    #[test]
    fn rocksdbtx() -> crate::Result<()> {
        let test = Test {
            name: "hello".to_string(),
            age: 10,
        };

        use crate::backend::rocksdb::RocksDbOptimistic;
        let backend = RocksDbOptimistic::new("database/rocks2")?;

        let env = Env::new(backend);
        let db = env.open_tupel::<(&str, SerdeJson<Test>)>("test")?;
        db.set("hello", &test)?;
        let res = db.get("hello")?;
        assert_eq!(res, Some(test));
        let env2 = env.clone();

        let handler = thread::spawn(move || {
            let test = Test {
                name: "hello".to_string(),
                age: 10,
            };
            let db = env2.open::<&str, SerdeJson<Test>>("test").unwrap();
            let db2 = db.clone();
            db2.set("hello", &test).unwrap();
            let res = db.get("hello").unwrap();
            assert_eq!(res, Some(test));
        });

        handler.join().unwrap();
        let db = env.open::<&str, SerdeJson<Test>>("test").unwrap();
        let _res = db.get("hello")?;
        let tx = db.transaction()?;
        tx.get("hello")?;
        tx.commit()?;
        db.delete_db()?;

        Ok(())
    }

    #[test]
    fn rocksdb() -> crate::Result<()> {
        let test = Test {
            name: "hello".to_string(),
            age: 10,
        };

        use crate::backend::rocksdb::RocksDb;
        let backend = RocksDb::new("database/rocks")?;

        let env = Env::new(backend);
        let db = env.open::<&str, SerdeJson<Test>>("test")?;
        db.set("hello", &test)?;
        let res = db.get("hello")?;
        assert_eq!(res, Some(test));

        let env2 = env.clone();
        let handler = thread::spawn(move || {
            let test = Test {
                name: "hello".to_string(),
                age: 10,
            };
            let db = env2.open::<&str, SerdeJson<Test>>("test").unwrap();
            let db2 = db.clone();
            db2.set("hello", &test).unwrap();
            let res = db.get("hello").unwrap();
            assert_eq!(res, Some(test));
        });

        handler.join().unwrap();
        let db = env.open::<&str, SerdeJson<Test>>("test").unwrap();
        let _res = db.get("hello")?;
        db.flush()?;

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
        let db = env.open::<&str, SerdeJson<Test>>("test")?;
        db.set("hello", &test)?;
        let res = db.get("hello")?;
        assert_eq!(res, Some(test));

        let env2 = env.clone();
        let handler = thread::spawn(move || {
            let test = Test {
                name: "hello".to_string(),
                age: 10,
            };
            let db = env2.open::<&str, SerdeJson<Test>>("test").unwrap();
            let db2 = db.clone();
            db2.set("hello", &test).unwrap();
            let res = db.get("hello").unwrap();
            assert_eq!(res, Some(test));
        });

        handler.join().unwrap();
        let db = env.open::<&str, SerdeJson<Test>>("test").unwrap();
        let _res = db.get("hello")?;
        Ok(())
    }
}
