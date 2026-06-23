//! File-backed local sync metadata store.
//!
//! This crate persists sync timestamps, known local resource hashes, and
//! pending operations. It does not read vault content, connect to the server,
//! or decide sync conflicts.

mod error;
mod pending;
mod state;
mod store;

pub use error::{LocalStoreError, Result};
pub use pending::{PendingDelete, PendingModify, PendingRename, PendingState};
pub use state::{HashEntry, LocalStoreState, SyncTimes};
pub use store::LocalStore;
