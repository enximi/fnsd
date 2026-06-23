use std::path::{Path, PathBuf};

use fns_core::ContentHash;
use fns_file_transfer::DownloadSession;

use crate::{Result, SyncEngineError};

pub(crate) struct DownloadCheckpointStore {
    root: PathBuf,
}

impl DownloadCheckpointStore {
    pub fn new(store_path: &Path) -> Self {
        let root = store_path
            .parent()
            .map(|parent| parent.join("download-checkpoints"))
            .unwrap_or_else(|| PathBuf::from("download-checkpoints"));
        Self { root }
    }

    pub fn restore(&self, session: &mut DownloadSession) -> Result<()> {
        let dir = self.session_dir(session);
        if !dir.exists() {
            return Ok(());
        }

        for chunk_index in 0..session.total_chunks() {
            let path = dir.join(format!("{chunk_index}.chunk"));
            if !path.exists() {
                continue;
            }

            let bytes = std::fs::read(&path).map_err(|err| io(&path, err))?;
            session.restore_chunk(chunk_index as u32, bytes)?;
        }

        Ok(())
    }

    pub fn save_chunk(
        &self,
        session: &DownloadSession,
        chunk_index: u32,
        chunk_data: &[u8],
    ) -> Result<()> {
        let dir = self.session_dir(session);
        std::fs::create_dir_all(&dir).map_err(|err| io(&dir, err))?;
        let path = dir.join(format!("{chunk_index}.chunk"));
        std::fs::write(&path, chunk_data).map_err(|err| io(&path, err))?;
        Ok(())
    }

    pub fn clear_completed(
        &self,
        content_hash: &ContentHash,
        size: u64,
        chunk_size: usize,
    ) -> Result<()> {
        let dir = self
            .root
            .join(checkpoint_key(content_hash, size, chunk_size));
        self.clear_dir(&dir)
    }

    fn clear_dir(&self, dir: &Path) -> Result<()> {
        if dir.exists() {
            std::fs::remove_dir_all(&dir).map_err(|err| io(&dir, err))?;
        }
        Ok(())
    }

    fn session_dir(&self, session: &DownloadSession) -> PathBuf {
        self.root.join(checkpoint_key(
            session.content_hash(),
            session.size(),
            session.chunk_size(),
        ))
    }
}

fn checkpoint_key(content_hash: &ContentHash, size: u64, chunk_size: usize) -> String {
    format!("{}-{size}-{chunk_size}", safe_name(content_hash.as_str()))
}

fn safe_name(value: &str) -> String {
    value
        .chars()
        .map(|ch| match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' => ch,
            _ => '_',
        })
        .collect()
}

fn io(path: &Path, err: std::io::Error) -> SyncEngineError {
    SyncEngineError::CheckpointIo {
        path: path.to_path_buf(),
        source: err,
    }
}
