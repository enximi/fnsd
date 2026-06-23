//! File chunk transfer helpers for the FNS headless client.
//!
//! This crate builds upload chunk frames, tracks download chunk sessions, and
//! verifies downloaded file content before writing it to the local vault. It
//! does not own WebSocket connections or decide when a transfer should start.

mod download;
mod error;
mod upload;

pub use download::{DownloadSession, DownloadedFile, build_file_get_request};
pub use error::{FileTransferError, Result};
pub use upload::{UploadPlan, build_upload_plan};

fn valid_positive_usize(value: i64, name: &'static str) -> Result<usize> {
    if value <= 0 {
        return Err(FileTransferError::InvalidPositiveNumber { name, value });
    }

    usize::try_from(value).map_err(|_| FileTransferError::NumberTooLarge { name, value })
}

fn valid_non_negative_u64(value: i64, name: &'static str) -> Result<u64> {
    if value < 0 {
        return Err(FileTransferError::InvalidNonNegativeNumber { name, value });
    }

    u64::try_from(value).map_err(|_| FileTransferError::NumberTooLarge { name, value })
}

fn expected_chunk_count(size: u64, chunk_size: usize) -> usize {
    if size == 0 {
        return 0;
    }

    size.div_ceil(chunk_size as u64) as usize
}
