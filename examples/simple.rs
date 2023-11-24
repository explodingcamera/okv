use eyre::Result;
use okv::backend::rocksdb::RocksDbOptimistic;
use okv::Env;

fn main() -> Result<()> {
    std::fs::create_dir_all("database/example-simple")?;
    let rocksdb = RocksDbOptimistic::new("database/example-simple")?;
    let env = Env::new(rocksdb);

    let db = env.open::<&str, &str>("test")?;
    db.set_nx("hello", "world")?;

    let res = db.get("hello")?;

    assert_eq!(res, Some("world".to_string()));
    Ok(())
}

#[test]
fn test() -> Result<()> {
    main()
}
