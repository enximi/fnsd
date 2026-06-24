//! 长期同步会话。
//!
//! 该模块保持一个 WebSocket 长连接，执行启动同步，
//! 并通过同一个连接发送本地增量变化。

mod echo;
mod error;
mod file_transfer;
mod local_change;
mod pending_watch;
mod session;

pub use error::{Result, SyncSessionError};
pub use session::{SyncSession, SyncSessionCommand};
