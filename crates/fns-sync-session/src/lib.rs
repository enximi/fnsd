//! Long-lived sync session.
//!
//! This crate keeps one WebSocket connection open, performs startup sync, and
//! sends incremental local changes over that same connection.

mod error;
mod local_change;
mod session;

pub use error::{Result, SyncSessionError};
pub use session::{SyncSession, SyncSessionCommand, SyncSessionOptions};
