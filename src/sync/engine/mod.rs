//! FNS headless 客户端的一次性同步编排。
//!
//! 该模块把配置、本地元数据、vault 文件系统访问、协议规划和 WebSocket 传输串起来。
//! 策略应在这里保持显式，底层工作交给更窄的模块。

mod checkpoint;
mod engine;
mod error;
mod event_loop;
mod outgoing;
mod snapshot;
mod transfer;

pub use engine::{MissingPathMode, SyncEngine, SyncEngineOptions, SyncOnceSummary};
pub use error::{Result, SyncEngineError};
pub use transfer::TransferOptions;
