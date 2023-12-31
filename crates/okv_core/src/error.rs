use thiserror::Error;

/// A specialized [`Result`] type for this crate.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// An error that can occur when interacting with the database.
#[derive(Error, Debug)]
pub enum Error {
    /// An error that can occur when decoding a value.
    #[error("Decode error: {0}")]
    Decode(#[from] DecodeError),

    /// An error that can occur when encoding a value.
    #[error("Encode error: {0}")]
    Encode(#[from] EncodeError),

    /// A key was not found in the database.
    #[error("Key not found: {key:?}")]
    KeyNotFound {
        /// The key that was not found.
        key: Vec<u8>,
    },

    /// No database by the given name was found.
    #[error("Database not found: {db}")]
    DatabaseNotFound {
        /// The database that was not found.
        db: String,
    },

    /// Database backend error.
    #[error("Database backend error: {0}")]
    DatabaseBackend(#[from] Box<dyn std::error::Error + Send + Sync>),

    /// An error that can occur when using the `rocksdb` crate.
    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// An error that can occur when decoding a value.
#[derive(Error, Debug)]
pub enum DecodeError {
    /// The given bytes are not valid UTF-8.
    #[error("Invalid UTF-8")]
    InvalidUtf8,

    /// An IO error occurred.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// The given bytes do not match the expected size.
    #[error("Size mismatch")]
    SizeMismatch,

    /// [`serde_json::Error`]
    #[cfg(feature = "serde_json")]
    #[error("Serde JSON error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    /// [`rmp_serde::decode::Error`]
    #[cfg(feature = "rmp-serde")]
    #[error("Serde RMP error: {0}")]
    SerdeRmp(#[from] rmp_serde::decode::Error),

    /// [`uuid::Error`]
    #[cfg(feature = "uuid")]
    #[error("UUID error: {0}")]
    Uuid(#[from] uuid::Error),
}

/// An error that can occur when encoding a value.
#[derive(Error, Debug)]
pub enum EncodeError {
    /// An IO error occurred.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// [`serde_json::Error`]
    #[cfg(feature = "serde_json")]
    #[error("Serde JSON error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    /// [`rmp_serde::encode::Error`]
    #[cfg(feature = "rmp-serde")]
    #[error("Serde RMP error: {0}")]
    SerdeRmp(#[from] rmp_serde::encode::Error),
}
