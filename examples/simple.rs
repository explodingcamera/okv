use eyre::Result;
use okv::backend::rocksdb::RocksDbOptimistic;
use okv::Env;
use okv_core::traits_async::DBCommonAsync;

fn main() -> Result<()> {
    // ensure that the directory exists
    std::fs::create_dir_all("database/example-simple")?;

    // initialize the storage backend
    let rocksdb = RocksDbOptimistic::new("database/example-simple")?;
    let env = Env::new(rocksdb);

    // open a database with the specified key and value types
    let db = env.open::<&str, &str>("test")?;

    db.async_set("hello", "world");

    db.set_nx("hello", "world")?;
    assert_eq!(db.get("hello")?, Some("world".to_string()));

    Ok(())
}

#[test] // ensure that the example always works
fn test() -> Result<()> {
    main()
}
