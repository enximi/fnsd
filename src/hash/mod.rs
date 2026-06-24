//! 与原 FNS Obsidian 插件兼容的 hash 辅助函数。
//!
//! 原插件使用 JavaScript 32 位滚动 hash。
//! 文本 hash 按 `charCodeAt` 语义计算，因此这里对 UTF-16 code unit 做 hash，
//! 而不是直接对 Rust char 做 hash。

pub(crate) mod bytes;
mod path;
mod text;

pub use bytes::file_content_hash;
pub use path::path_hash;
pub use text::text_content_hash;

fn finish_hash(value: i32) -> String {
    value.to_string()
}
