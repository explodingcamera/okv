use eyre::Result;
use okv::backend::memory::MemDB;
use okv::types::serde::SerdeJson;
use okv::Env;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Person {
    name: String,
    age: u32,
}

fn main() -> Result<()> {
    // initialize the storage backend
    let memdb = MemDB::new();
    let env = Env::new(memdb);

    // open a database with the specified key and value types
    let db = env.open::<&str, SerdeJson<Person>>("people")?;

    // insert a new person
    db.set_nx(
        "john",
        &Person {
            name: "John Doe".to_string(),
            age: 42,
        },
    )?;

    // retrieve the person
    let person = db.get("john")?;

    assert_eq!(
        person,
        Some(Person {
            name: "John Doe".to_string(),
            age: 42,
        })
    );

    // retrieve the raw value
    let raw = db.get_raw("john")?;
    assert_eq!(raw, Some(b"{\"name\":\"John Doe\",\"age\":42}".to_vec()));

    Ok(())
}

#[test] // ensure that the example always works
fn test() -> Result<()> {
    main()
}
