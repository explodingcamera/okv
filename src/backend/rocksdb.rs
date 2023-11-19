use super::DatabaseBackend;

pub struct RocksDb {
    db: rocksdb::DB,
}

impl RocksDb {
    pub fn new(connect_str: &str) -> Self {
        let db = rocksdb::DB::open_default(connect_str).unwrap();
        Self { db }
    }
}

impl DatabaseBackend for RocksDb {
    fn create_or_open(&self, db: &str) -> crate::Result<()> {
        self.db.create_cf(db, &rocksdb::Options::default()).unwrap();
        self.db.get_cf(&db, key)?;
        Ok(())
    }
}
