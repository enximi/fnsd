//! 与原 FNS Obsidian 插件兼容的 hash 辅助函数。
//!
//! 原插件使用 JavaScript 32 位滚动 hash。
//! 文本 hash 按 `charCodeAt` 语义计算，因此这里对 UTF-16 code unit 做 hash，
//! 而不是直接对 Rust char 做 hash。

mod bytes;
mod path;
mod text;

pub use bytes::{
    FILE_HASH_SLICE_SIZE, FILE_HASH_THRESHOLD, binary_content_hash, file_content_hash,
    file_content_hash_parts,
};
pub use path::path_hash;
pub use text::{text_content_hash, text_hash_value};

fn finish_hash(value: i32) -> String {
    value.to_string()
}
