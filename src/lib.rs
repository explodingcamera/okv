mod error;
mod traits;
pub mod types;

mod backend;
mod db;
mod env;

pub use error::*;
pub type Result<T> = std::result::Result<T, Error>;

pub use backend::mem;
pub use db::Database;
pub use env::Env;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
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
    fn test() -> crate::Result<()> {
        let test = Test {
            name: "hello".to_string(),
            age: 10,
        };

        let backend = MemDB::new();
        let env = Env::new(backend);
        let mut db = env.open::<&str, SerdeJson<Test>>("test")?;
        db.set("hello", &test)?;
        let res = db.get("hello")?;
        dbg!(res);

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
            println!("{:?}", res);
        });

        handler.join().unwrap();
        let db = env.open::<&str, SerdeJson<Test>>("test").unwrap();
        let res = db.get("hello")?;
        println!("{:?}", res);
        Ok(())
    }
}
