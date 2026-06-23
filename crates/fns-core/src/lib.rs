//! Shared domain types for the FNS headless client.
//!
//! This crate intentionally avoids filesystem, network, CLI, and protocol
//! behavior. It should stay small and easy to test.

mod error;
mod hash;
mod path;
mod resource;
mod time;

pub use error::{CoreError, Result};
pub use hash::{ContentHash, PathHash};
pub use path::{VaultName, VaultPath};
pub use resource::{ResourceId, ResourceKind};
pub use time::{RemoteMillis, UnixSeconds};
