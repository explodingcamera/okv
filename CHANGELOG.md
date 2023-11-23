# Changelog

## v0.1.0 - 2023-11-23 

* Database now works without specifying a lifetime parameter
  > This is done by using a `self_cell` to store the associated environment
* Remove `mut` requirement for all methods

## v0.0.8 - 2023-11-23

* Add `_raw` methods to `Database` for raw access to the underlying database
* Added `uuid` feature for `Uuid` support
* Properly implement `get_multi` (was previously just a loop over `get`)
* Removed lifetime parameters from `Env`

Full Changelog: https://github.com/explodingcamera/okv/commits/v0.0.8

## v0.0.7 - 2023-11-22

* Added support for transactions
* Refactored Database Traits
* Added `Env::open_tupel` method for opening a database with a tuple of types
* Added `Env::open_lazy` method for opening a database with a lazy type (as a workaround for Database not bein Sync)
* RocksDB backend now supports + Sync

Full Changelog: https://github.com/explodingcamera/okv/commits/v0.0.7

## v0.0.6 - 2023-11-22

* Added `DatabaseColumnTxn` trait for transactional columns
* Added `DatabaseTxn` trait for transactions
* Renamed `DatabaseColumn` trait to `DatabaseCommon`
* Moved `get_multi_ref` to `DatabaseCommonRefMut` trait
* Added `Database::delete_db` method for deleting the underlying database
* Removed `DatabaseCommonClear` implementation for RocksDB backends and replaced with `DatabaseCommonDelete`
* `Database::inner` now returns a reference to the underlying column instead of the database driver

Full Changelog: https://github.com/explodingcamera/okv/commits/v0.0.6

## v0.0.5 - 2023-11-21

* Less lifetime parameters
* Added `backend::rocksdb::RocksDBOptimistic` for optimistic locking
* Added `backend::rocksdb::RocksDBPessimistic` for pessimistic locking
* Export `Flushable` and `Innerable` traits

Full Changelog: https://github.com/explodingcamera/okv/commits/v0.0.5

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