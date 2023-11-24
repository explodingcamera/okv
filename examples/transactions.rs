use eyre::Result;
use okv::backend::rocksdb::RocksDbOptimistic;
use okv::Env;

fn main() -> Result<()> {
    // ensure that the directory exists
    std::fs::create_dir_all("database/example-transactions")?;

    // initialize the storage backend
    // You can either use RocksDbOptimistic or RocksDbPessimistic
    // see https://github.com/facebook/rocksdb/wiki/Transactions
    let rocksdb = RocksDbOptimistic::new("database/example-transactions")?;
    let env = Env::new(rocksdb);

    // open a database with the specified key and value types
    let db = env.open::<&str, &str>("test")?;

    let tx = db.transaction()?;
    tx.set_nx("hello", "world")?;
    tx.commit()?;

    // after commit, we can't use tx anymore as it has been consumed
    // tx.set_nx("hello", "world3")?; <- borrow of moved value: `tx`

    let tx = db.transaction()?;
    tx.set("hello", "sailor")?;
    tx.rollback()?;

    // hello is still world as the transaction was rolled back
    assert_eq!(db.get("hello")?, Some("world".to_string()));

    Ok(())
}

#[test] // ensure that the example always works
fn test() -> Result<()> {
    main()
}
