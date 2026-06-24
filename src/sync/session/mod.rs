//! 长期同步会话。
//!
//! 该模块保持一个 WebSocket 长连接，执行启动同步，
//! 并通过同一个连接发送本地增量变化。

mod error;
mod local_change;
mod session;

pub use error::{Result, SyncSessionError};
pub use session::{SyncSession, SyncSessionCommand, SyncSessionOptions};
