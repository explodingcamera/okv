# Changelog

## v0.0.5 - 2023-11-21

* Less lifetime parameters
* Added `backend::rocksdb::RocksDBOptimistic` for optimistic locking
* Added `backend::rocksdb::RocksDBPessimistic` for pessimistic locking
* Export `Flushable` and `Innerable` traits

## v0.0.4 - 2023-11-21

* added `Database::get_multi`, `Database::clear`, `Database::remove`,`Database::contains`, `Database::flush` methods
* Refactored `Database` to use the new DBCommon trait
* Added `.inner()` methods to `Env` and `Database` to allow access to the underlying database
* Experimental `AnyDatabase` to allow for multiple database types to be used in the same application 

Full Changelog: https://github.com/explodingcamera/okv/commits/v0.0.4

## v0.0.3 - 2023-11-20

* Improved Documentation (README/Rust Docs)
* Added bool support for serialization
* Fix feature flags

Full Changelog: https://github.com/explodingcamera/okv/commits/v0.0.3