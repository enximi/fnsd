//! FNS vault 的本地文件系统适配层。
//!
//! 该模块负责路径解析、扫描和基础 vault 文件操作。
//! 它不编码协议 frame，不持久化同步状态，也不判断冲突。

mod error;
mod scan;
mod vault;

pub use error::{Result, VaultFsError};
pub use scan::{ScanRule, VaultScanOptions, VaultSnapshot};
pub use vault::{VaultFileTimes, VaultFs};
