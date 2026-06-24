use crate::core::{ContentHash, VaultPath};

use crate::store::Result;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UploadCheckpoint {
    pub session_id: String,
    pub content_hash: String,
    pub last_chunk_index: u32,
}

impl UploadCheckpoint {
    pub fn new(
        session_id: impl Into<String>,
        content_hash: ContentHash,
        last_chunk_index: u32,
    ) -> Self {
        Self {
            session_id: session_id.into(),
            content_hash: content_hash.into_string(),
            last_chunk_index,
        }
    }

    pub fn content_hash(&self) -> Result<ContentHash> {
        Ok(ContentHash::new(&self.content_hash)?)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingRename {
    pub old_path: String,
    pub new_path: String,
    pub content_hash: Option<String>,
}

impl PendingRename {
    pub fn new(
        old_path: VaultPath,
        new_path: VaultPath,
        content_hash: Option<ContentHash>,
    ) -> Self {
        Self {
            old_path: old_path.into_string(),
            new_path: new_path.into_string(),
            content_hash: content_hash.map(ContentHash::into_string),
        }
    }

    pub fn old_path(&self) -> Result<VaultPath> {
        Ok(VaultPath::new(&self.old_path)?)
    }

    pub fn new_path(&self) -> Result<VaultPath> {
        Ok(VaultPath::new(&self.new_path)?)
    }

    pub fn content_hash(&self) -> Result<Option<ContentHash>> {
        Ok(self
            .content_hash
            .as_ref()
            .map(ContentHash::new)
            .transpose()?)
    }
}
