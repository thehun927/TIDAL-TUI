pub mod auth;
pub mod catalog;
pub mod client;
pub mod collections;
pub mod models;
pub mod playback;
pub mod recommendations;
pub mod search;

// Convenience re-exports used across the codebase.
pub use client::TidalClient;
#[allow(unused_imports)]
pub use models::AudioQuality;
