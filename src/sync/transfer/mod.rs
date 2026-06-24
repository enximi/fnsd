//! FNS headless 客户端的文件分片传输辅助模块。
//!
//! 该模块构造上传分片 frame，跟踪下载分片会话，并在写入本地 vault 前校验下载内容。
//! 它不持有 WebSocket 连接，也不决定传输何时开始。

mod download;
mod error;
mod upload;

pub use download::{DownloadSession, build_file_get_request};
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
