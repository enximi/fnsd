//! FNS headless 客户端的长期运行调度器。
//!
//! 该模块负责长期运行、watch 转发和 session 重连退避。
//! 它不构造协议消息，不应用服务端事件，也不直接传输文件分片。

mod daemon;
mod error;
mod watch_task;

pub use daemon::Daemon;
pub use error::{DaemonError, Result};
