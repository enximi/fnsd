//! FNS headless 客户端的 WebSocket 传输层。
//!
//! 该模块负责发送和接收协议 frame。
//! 它不扫描本地文件，不判断同步冲突，不持久化状态，也不实现重连策略。

mod client;
mod error;
mod event;

pub use client::{ClientDescriptor, WebSocketClient};
pub use error::{Result, WsClientError};
pub use event::WsEvent;
