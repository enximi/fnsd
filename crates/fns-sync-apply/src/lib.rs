//! Apply FNS sync events to the local vault and local sync metadata.
//!
//! This crate translates protocol text events into local side effects on
//! `VaultFs` and `LocalStore`. It does not open sockets, scan vaults, build
//! sync requests, or run the sync lifecycle.

mod dispatch;
mod error;
mod file;
mod folder;
mod local;
mod note;
mod outcome;
mod sent;
mod setting;

pub use dispatch::apply_text_event;
pub use error::{Result, SyncApplyError};
pub use outcome::{EventApplySummary, EventOutcome, SyncEndTracker, pending_sync_end_events};
pub use sent::refresh_sent_hash_index;
