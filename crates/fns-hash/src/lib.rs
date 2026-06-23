//! Hash helpers compatible with the original FNS Obsidian plugin.
//!
//! The plugin uses JavaScript 32-bit rolling hashes. Text hashing follows
//! `charCodeAt`, so this crate hashes UTF-16 code units instead of Rust chars.

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
