use crate::core::{ContentHash, VaultPath};

pub type Result<T> = std::result::Result<T, FileTransferError>;

#[derive(Debug, thiserror::Error)]
pub enum FileTransferError {
    #[error(transparent)]
    Core(#[from] crate::core::CoreError),
    #[error(transparent)]
    Protocol(#[from] crate::protocol::ProtocolError),
    #[error(transparent)]
    VaultFs(#[from] crate::vault::fs::VaultFsError),
    #[error("{name} must be positive, got {value}")]
    InvalidPositiveNumber { name: &'static str, value: i64 },
    #[error("{name} must be non-negative, got {value}")]
    InvalidNonNegativeNumber { name: &'static str, value: i64 },
    #[error("{name} is too large: {value}")]
    NumberTooLarge { name: &'static str, value: i64 },
    #[error(
        "download total_chunks does not match size/chunk_size: expected {expected}, got {actual}"
    )]
    TotalChunksMismatch { expected: usize, actual: usize },
    #[error("file chunk session mismatch: expected {expected}, got {actual}")]
    SessionMismatch { expected: String, actual: String },
    #[error("chunk index out of range: index {index}, total {total}")]
    ChunkIndexOutOfRange { index: u32, total: usize },
    #[error("duplicate chunk index: {0}")]
    DuplicateChunk(u32),
    #[error("missing chunk index: {0}")]
    MissingChunk(usize),
    #[error("file size mismatch for {path}: expected {expected}, got {actual}")]
    SizeMismatch {
        path: VaultPath,
        expected: u64,
        actual: u64,
    },
    #[error("content hash mismatch for {path}: expected {expected}, got {actual}")]
    ContentHashMismatch {
        path: VaultPath,
        expected: ContentHash,
        actual: ContentHash,
    },
    #[error(
        "upload checkpoint chunk index {index} is out of range for {path}: total chunks {total}"
    )]
    UploadCheckpointOutOfRange {
        path: VaultPath,
        index: u32,
        total: usize,
    },
}
