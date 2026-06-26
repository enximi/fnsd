use crate::core::ContentHash;

use crate::hash::finish_hash;

pub const FILE_HASH_THRESHOLD: usize = 10 * 1024 * 1024;
pub const FILE_HASH_SLICE_SIZE: usize = 5 * 1024 * 1024;

pub fn binary_content_hash(bytes: &[u8]) -> ContentHash {
    ContentHash::new(bytes_hash_value(bytes)).expect("computed binary hash must be valid")
}

pub fn setting_content_hash(bytes: &[u8]) -> ContentHash {
    binary_content_hash(bytes)
}

pub fn file_content_hash(bytes: &[u8]) -> ContentHash {
    if bytes.len() <= FILE_HASH_THRESHOLD {
        return binary_content_hash(bytes);
    }

    let head = &bytes[..FILE_HASH_SLICE_SIZE];
    let tail = &bytes[bytes.len() - FILE_HASH_SLICE_SIZE..];

    file_content_hash_parts(head, tail)
}

pub fn file_content_hash_parts(head: &[u8], tail: &[u8]) -> ContentHash {
    let mut hasher = RollingByteHash::default();
    hasher.update(pad_or_truncate(head, FILE_HASH_SLICE_SIZE));
    hasher.update(pad_or_truncate(tail, FILE_HASH_SLICE_SIZE));
    ContentHash::new(finish_hash(hasher.finish())).expect("computed file hash must be valid")
}

fn bytes_hash_value(bytes: &[u8]) -> String {
    let mut hasher = RollingByteHash::default();
    hasher.update(bytes.iter().copied());
    finish_hash(hasher.finish())
}

fn pad_or_truncate(bytes: &[u8], len: usize) -> impl Iterator<Item = u8> + '_ {
    bytes
        .iter()
        .copied()
        .take(len)
        .chain(std::iter::repeat_n(0, len.saturating_sub(bytes.len())))
}

#[derive(Default)]
struct RollingByteHash {
    value: i32,
}

impl RollingByteHash {
    fn update(&mut self, bytes: impl IntoIterator<Item = u8>) {
        for byte in bytes {
            self.value = self.value.wrapping_shl(5).wrapping_sub(self.value);
            self.value = self.value.wrapping_add(i32::from(byte));
        }
    }

    fn finish(self) -> i32 {
        self.value
    }
}
