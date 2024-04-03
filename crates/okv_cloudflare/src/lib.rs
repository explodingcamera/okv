#[cfg(feature = "worker")]
/// Cloudflare Worker API based backends
/// This is only safe to use in Cloudflare Workers and will not work in any other environment.
pub mod worker;

// pub mod http;
