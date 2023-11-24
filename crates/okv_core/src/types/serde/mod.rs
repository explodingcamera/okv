#[cfg(feature = "serde-json")]
mod json;
#[cfg(feature = "serde-json")]
pub use self::json::SerdeJson;

#[cfg(feature = "serde-rmp")]
mod rmp;
#[cfg(feature = "serde-rmp")]
pub use self::rmp::SerdeRmp;
