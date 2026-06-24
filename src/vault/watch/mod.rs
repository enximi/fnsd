//! Vault 文件系统监听器。
//!
//! 该模块把操作系统文件通知归一化为 vault 级别的变化信号。
//! 它不执行同步，不连接服务器，也不持久化状态。

mod error;
mod event;
mod watcher;

pub use error::{Result, VaultWatchError};
pub use event::VaultWatchEvent;
pub use watcher::VaultWatcher;
