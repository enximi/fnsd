//! 基于文件的本地同步元数据存储。
//!
//! 该模块持久化同步时间戳、已知本地资源 hash 和 pending 操作。
//! 它不读取 vault 内容，不连接服务器，也不判断同步冲突。

mod error;
mod pending;
mod state;
mod store;

pub use error::{LocalStoreError, Result};
pub use pending::{PendingDelete, PendingModify, PendingRename, PendingState, UploadCheckpoint};
pub use state::{HashEntry, LocalStoreState, SyncTimes};
pub use store::LocalStore;
