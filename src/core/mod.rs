//! FNS headless 客户端共享的领域类型。
//!
//! 该模块刻意不包含文件系统、网络、CLI 和协议行为。
//! 它应该保持小而稳定。

mod error;
mod hash;
mod path;
mod resource;
mod time;

pub use error::{CoreError, Result};
pub use hash::{ContentHash, PathHash};
pub use path::{VaultName, VaultPath};
pub use resource::{
    DeletedResource, FileResource, FolderResource, NoteResource, ResourceId, ResourceKind,
    SettingResource, SyncBatch, TextResource,
};
pub use time::{RemoteMillis, UnixSeconds};
