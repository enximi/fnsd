//! Configuration loading and validation for the FNS headless client.
//!
//! This crate reads TOML configuration, applies environment overrides, and
//! validates values. It does not open WebSocket connections or touch vault
//! content.

mod error;
mod model;

pub use error::{ConfigError, Result};
pub use model::{
    AppConfig, ClientConfig, RuleConfig, ScanConfig, ServerConfig, StoreConfig, SyncConfig,
    VaultConfig,
};
