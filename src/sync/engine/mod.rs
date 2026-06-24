//! fnsd 的一次性同步编排。
//!
//! 该模块把配置、本地元数据、vault 文件系统访问、协议规划和 WebSocket 传输串起来。
//! 策略应在这里保持显式，底层工作交给更窄的模块。

mod error;
mod event_loop;
mod outgoing;
mod snapshot;
mod sync_once;
mod transfer_queue;

pub use error::{Result, SyncEngineError};
pub use sync_once::{MissingPathMode, SyncEngine};
pub use transfer_queue::TransferOptions;
