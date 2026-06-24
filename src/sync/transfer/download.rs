use crate::core::{ContentHash, RemoteMillis, VaultName, VaultPath};
use crate::hash::file_content_hash;
use crate::protocol::{FileChunkFrame, FileGetRequest};
use crate::sync::plan::{FileDownload, RemoteFile};
use crate::vault::fs::{VaultFileTimes, VaultFs};

use crate::sync::transfer::{
    FileTransferError, Result, expected_chunk_count, valid_non_negative_u64, valid_positive_usize,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadSession {
    path: VaultPath,
    content_hash: ContentHash,
    ctime: RemoteMillis,
    mtime: RemoteMillis,
    session_id: String,
    chunk_size: usize,
    size: u64,
    chunks: Vec<Option<Vec<u8>>>,
}

impl DownloadSession {
    pub fn new(download: FileDownload) -> Result<Self> {
        let chunk_size = valid_positive_usize(download.chunk_size, "chunk_size")?;
        let total_chunks = valid_non_negative_u64(download.total_chunks, "total_chunks")?;
        let total_chunks =
            usize::try_from(total_chunks).map_err(|_| FileTransferError::NumberTooLarge {
                name: "total_chunks",
                value: download.total_chunks,
            })?;
        let size = valid_non_negative_u64(download.size, "size")?;
        let expected = expected_chunk_count(size, chunk_size);

        if total_chunks != expected {
            return Err(FileTransferError::TotalChunksMismatch {
                expected,
                actual: total_chunks,
            });
        }

        Ok(Self {
            path: download.path,
            content_hash: download.content_hash,
            ctime: download.ctime,
            mtime: download.mtime,
            session_id: download.session_id,
            chunk_size,
            size,
            chunks: vec![None; total_chunks],
        })
    }

    pub fn path(&self) -> &VaultPath {
        &self.path
    }

    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    pub fn content_hash(&self) -> &ContentHash {
        &self.content_hash
    }

    pub fn chunk_size(&self) -> usize {
        self.chunk_size
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn total_chunks(&self) -> usize {
        self.chunks.len()
    }

    pub fn received_chunks(&self) -> usize {
        self.chunks.iter().filter(|chunk| chunk.is_some()).count()
    }

    pub fn is_complete(&self) -> bool {
        self.received_chunks() == self.total_chunks()
    }

    pub fn accept_chunk(&mut self, chunk: FileChunkFrame) -> Result<()> {
        if chunk.session_id() != self.session_id {
            return Err(FileTransferError::SessionMismatch {
                expected: self.session_id.clone(),
                actual: chunk.session_id().to_string(),
            });
        }

        let index = chunk.chunk_index();
        let Some(slot) = self.chunks.get_mut(index as usize) else {
            return Err(FileTransferError::ChunkIndexOutOfRange {
                index,
                total: self.chunks.len(),
            });
        };

        if let Some(existing) = slot {
            if existing == chunk.chunk_data() {
                return Ok(());
            }

            return Err(FileTransferError::DuplicateChunk(index));
        }

        *slot = Some(chunk.chunk_data().to_vec());
        Ok(())
    }

    pub fn restore_chunk(&mut self, chunk_index: u32, chunk_data: Vec<u8>) -> Result<()> {
        let chunk = FileChunkFrame::new(&self.session_id, chunk_index, chunk_data)?;
        self.accept_chunk(chunk)
    }

    pub fn finalize(self, vault: &VaultFs) -> Result<DownloadedFile> {
        let mut bytes = Vec::with_capacity(self.size as usize);

        for (index, chunk) in self.chunks.into_iter().enumerate() {
            let Some(chunk) = chunk else {
                return Err(FileTransferError::MissingChunk(index));
            };
            bytes.extend_from_slice(&chunk);
        }

        let actual_size = bytes.len() as u64;
        if actual_size != self.size {
            return Err(FileTransferError::SizeMismatch {
                path: self.path,
                expected: self.size,
                actual: actual_size,
            });
        }

        let actual_hash = file_content_hash(&bytes);
        if actual_hash != self.content_hash {
            return Err(FileTransferError::ContentHashMismatch {
                path: self.path,
                expected: self.content_hash,
                actual: actual_hash,
            });
        }

        vault.write_bytes(
            &self.path,
            &bytes,
            Some(VaultFileTimes::new(Some(self.ctime), Some(self.mtime))),
        )?;

        Ok(DownloadedFile {
            path: self.path,
            content_hash: actual_hash,
            size: actual_size,
            mtime: self.mtime,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadedFile {
    pub path: VaultPath,
    pub content_hash: ContentHash,
    pub size: u64,
    pub mtime: RemoteMillis,
}

pub fn build_file_get_request(vault: &VaultName, file: &RemoteFile) -> FileGetRequest {
    FileGetRequest {
        vault: vault.to_string(),
        path: file.path.to_string(),
        path_hash: file.path_hash.to_string(),
    }
}
