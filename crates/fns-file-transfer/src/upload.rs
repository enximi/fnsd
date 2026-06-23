use fns_core::{ContentHash, RemoteMillis, VaultPath};
use fns_hash::file_content_hash;
use fns_protocol::FileChunkFrame;
use fns_sync_plan::FileUpload;
use fns_vault_fs::VaultFs;

use crate::{Result, valid_positive_usize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UploadPlan {
    pub path: VaultPath,
    pub session_id: String,
    pub content_hash: ContentHash,
    pub chunk_size: usize,
    pub total_chunks: usize,
    pub size: u64,
    pub mtime: RemoteMillis,
    pub chunks: Vec<FileChunkFrame>,
}

pub fn build_upload_plan(vault: &VaultFs, upload: &FileUpload) -> Result<UploadPlan> {
    let chunk_size = valid_positive_usize(upload.chunk_size, "chunk_size")?;
    let bytes = vault.read_bytes(&upload.path)?;
    let metadata = vault.file_metadata(&upload.path)?;
    let content_hash = file_content_hash(&bytes);
    let chunks = if bytes.is_empty() {
        vec![FileChunkFrame::new(&upload.session_id, 0, Vec::new())?]
    } else {
        bytes
            .chunks(chunk_size)
            .enumerate()
            .map(|(index, chunk)| FileChunkFrame::new(&upload.session_id, index as u32, chunk))
            .collect::<std::result::Result<Vec<_>, _>>()?
    };
    let total_chunks = chunks.len();

    Ok(UploadPlan {
        path: upload.path.clone(),
        session_id: upload.session_id.clone(),
        content_hash,
        chunk_size,
        total_chunks,
        size: bytes.len() as u64,
        mtime: metadata.mtime,
        chunks,
    })
}
