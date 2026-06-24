//! Vault filesystem watcher.
//!
//! This crate normalizes operating-system file notifications into vault-level
//! change signals. It does not run sync, connect to the server, persist state,
//! or infer rename semantics.

mod error;
mod event;
mod watcher;

pub use error::{Result, VaultWatchError};
pub use event::VaultWatchEvent;
pub use watcher::VaultWatcher;
