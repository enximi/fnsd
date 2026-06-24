//! 把 FNS 同步事件应用到本地 vault 和本地同步元数据。
//!
//! 该模块把协议文本事件转换成对 `VaultFs` 和 `LocalStore` 的本地副作用。
//! 它不打开 socket，不扫描 vault，不构造同步请求，也不运行同步生命周期。

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
