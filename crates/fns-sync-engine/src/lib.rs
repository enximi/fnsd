//! Sync orchestration for the FNS headless client.
//!
//! This crate wires configuration, local metadata, vault filesystem access,
//! protocol planning, and WebSocket transport together. It should keep policy
//! explicit and delegate low-level work to the narrower crates.

mod checkpoint;
mod engine;
mod error;
mod event_loop;
mod outgoing;
mod snapshot;
mod transfer;

pub use engine::{MissingPathMode, SyncEngine, SyncEngineOptions, SyncOnceSummary};
pub use error::{Result, SyncEngineError};
pub use transfer::TransferOptions;
