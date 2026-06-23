//! Local filesystem adapter for an FNS vault.
//!
//! This crate owns path resolution, scanning, and basic vault file operations.
//! It does not encode protocol frames, persist sync state, or decide conflicts.

mod error;
mod scan;
mod vault;

pub use error::{Result, VaultFsError};
pub use scan::{ScanRule, VaultScanOptions, VaultSnapshot};
pub use vault::{VaultFileTimes, VaultFs};
