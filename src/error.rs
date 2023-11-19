use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

pub trait ResultExt<T> {
    fn unwrap_not_found(self) -> Result<Option<T>>;
}

impl<T> ResultExt<T> for Result<T> {
    fn unwrap_not_found(self) -> Result<Option<T>> {
        match self {
            Ok(v) => Ok(Some(v)),
            Err(Error::KeyNotFound { .. }) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Decode error: {0}")]
    Decode(#[from] DecodeError),
    #[error("Encode error: {0}")]
    Encode(#[from] EncodeError),

    #[error("Key not found: {key:?}")]
    KeyNotFound { key: Vec<u8> },

    #[error("Database not found: {db}")]
    DatabaseNotFound { db: String },

    #[error("Database locked: {db}")]
    DatabaseLocked { db: String },

    #[cfg(feature = "rocksdb")]
    #[error("RocksDB error: {0}")]
    RocksDB(#[from] rocksdb::Error),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

#[derive(Error, Debug)]
pub enum DecodeError {
    #[error("Invalid UTF-8")]
    InvalidUtf8,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Size mismatch")]
    SizeMismatch,

    #[cfg(feature = "serde-json")]
    #[error("Serde JSON error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[cfg(feature = "serde-rmp")]
    #[error("Serde RMP error: {0}")]
    SerdeRmp(#[from] rmp_serde::decode::Error),
}

#[derive(Error, Debug)]
pub enum EncodeError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[cfg(feature = "serde-json")]
    #[error("Serde JSON error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[cfg(feature = "serde-rmp")]
    #[error("Serde RMP error: {0}")]
    SerdeRmp(#[from] rmp_serde::encode::Error),
}
