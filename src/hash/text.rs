use crate::core::ContentHash;

use crate::hash::finish_hash;

pub fn text_hash_value(content: &str) -> String {
    let mut hash = 0_i32;

    for code_unit in content.encode_utf16() {
        hash = hash.wrapping_shl(5).wrapping_sub(hash);
        hash = hash.wrapping_add(i32::from(code_unit));
    }

    finish_hash(hash)
}

pub fn text_content_hash(content: &str) -> ContentHash {
    ContentHash::new(text_hash_value(content)).expect("computed text hash must be valid")
}
