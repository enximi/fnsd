//! fnsd 的配置加载和校验。
//!
//! 该模块读取 TOML 配置，应用环境变量覆盖，并校验配置值。
//! 它不建立 WebSocket 连接，也不读取或修改 vault 内容。

mod error;
mod model;

pub use error::{ConfigError, Result};
pub use model::AppConfig;
