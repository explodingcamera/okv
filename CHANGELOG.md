# Changelog

## Unreleased

* added `Database::get_multi`, `Database::clear`, `Database::remove`,`Database::contains`, `Database::flush` methods
* Refactored `Database` to use the new DBCommon trait
* Added `.inner()` methods to `Env` and `Database` to allow access to the underlying database
* Experimental `AnyDatabase` to allow for multiple database types to be used in the same application 

## v0.0.3 - 2023-11-20

* Improved Documentation (README/Rust Docs)
* Added bool support for serialization
* Fix feature flags

Full Changelog: https://github.com/explodingcamera/okv/commits/v0.0.3