//! Long-running daemon scheduler for the FNS headless client.
//!
//! This crate coordinates repeated `sync_once` runs. It does not build
//! protocol messages, apply server events, or transfer file chunks directly.

mod daemon;
mod error;
mod watcher;

pub use daemon::{Daemon, DaemonOptions};
pub use error::{DaemonError, Result};
