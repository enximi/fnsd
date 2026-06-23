//! WebSocket transport for the FNS headless client.
//!
//! This crate sends and receives protocol frames. It does not scan local files,
//! decide sync conflicts, persist state, or implement reconnect policy.

mod client;
mod error;
mod event;

pub use client::{ClientInfo, FnsWsClient};
pub use error::{Result, WsClientError};
pub use event::WsEvent;
