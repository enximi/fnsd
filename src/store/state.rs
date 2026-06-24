use crate::core::{ContentHash, RemoteMillis};

use crate::store::Result;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HashEntry {
    pub content_hash: Option<String>,
    pub mtime: i64,
    pub size: u64,
}

impl HashEntry {
    pub fn new(content_hash: Option<ContentHash>, mtime: RemoteMillis, size: u64) -> Self {
        Self {
            content_hash: content_hash.map(ContentHash::into_string),
            mtime: mtime.as_i64(),
            size,
        }
    }

    pub fn content_hash(&self) -> Result<Option<ContentHash>> {
        Ok(self
            .content_hash
            .as_ref()
            .map(ContentHash::new)
            .transpose()?)
    }

    pub fn mtime(&self) -> Result<RemoteMillis> {
        Ok(RemoteMillis::new(self.mtime)?)
    }
}
