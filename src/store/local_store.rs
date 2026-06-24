use std::path::Path;

use crate::core::{ContentHash, RemoteMillis, ResourceKind, VaultPath};
use crate::sync::transfer::DownloadSession;
use rusqlite::Connection;

use crate::store::{HashEntry, PendingRename, Result, UploadCheckpoint, database, error::io};

#[derive(Debug)]
pub struct LocalStore {
    conn: Connection,
}

impl LocalStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|err| io(parent, err))?;
        }

        let conn = Connection::open(&path)?;
        database::initialize_schema(&conn)?;

        Ok(Self { conn })
    }

    pub fn sync_time(&self, kind: ResourceKind) -> Result<RemoteMillis> {
        database::sync_time(&self.conn, kind)
    }

    pub fn set_sync_time(&self, kind: ResourceKind, value: RemoteMillis) -> Result<()> {
        database::set_sync_time(&self.conn, kind, value)
    }

    pub fn hash_entry(&self, kind: ResourceKind, path: &VaultPath) -> Result<Option<HashEntry>> {
        database::hash_entry(&self.conn, kind, path)
    }

    pub fn set_hash_entry(
        &self,
        kind: ResourceKind,
        path: &VaultPath,
        entry: HashEntry,
    ) -> Result<()> {
        database::set_hash_entry(&self.conn, kind, path, &entry)
    }

    pub fn set_content_hash(
        &self,
        kind: ResourceKind,
        path: &VaultPath,
        content_hash: Option<ContentHash>,
        mtime: RemoteMillis,
        size: u64,
    ) -> Result<()> {
        self.set_hash_entry(kind, path, HashEntry::new(content_hash, mtime, size))
    }

    pub fn remove_hash_entry(
        &self,
        kind: ResourceKind,
        path: &VaultPath,
    ) -> Result<Option<HashEntry>> {
        database::remove_hash_entry(&self.conn, kind, path)
    }

    pub fn rename_hash_tree(&mut self, old_path: &VaultPath, new_path: &VaultPath) -> Result<()> {
        database::rename_hash_tree(&mut self.conn, old_path, new_path)
    }

    pub fn all_hash_paths(&self, kind: ResourceKind) -> Result<Vec<VaultPath>> {
        database::all_hash_paths(&self.conn, kind)
    }

    pub fn set_pending_modify(
        &self,
        kind: ResourceKind,
        path: &VaultPath,
        content_hash: &ContentHash,
    ) -> Result<()> {
        database::set_pending_modify(&self.conn, kind, path, content_hash)
    }

    pub fn remove_pending_modify(
        &self,
        kind: ResourceKind,
        path: &VaultPath,
    ) -> Result<Option<String>> {
        database::remove_pending_modify(&self.conn, kind, path)
    }

    pub fn has_pending_modify(&self, kind: ResourceKind, path: &VaultPath) -> Result<bool> {
        database::has_pending_modify(&self.conn, kind, path)
    }

    pub fn file_upload_checkpoint(&self, path: &VaultPath) -> Result<Option<UploadCheckpoint>> {
        database::file_upload_checkpoint(&self.conn, path)
    }

    pub fn set_file_upload_checkpoint(
        &self,
        path: &VaultPath,
        checkpoint: UploadCheckpoint,
    ) -> Result<()> {
        database::set_file_upload_checkpoint(&self.conn, path, &checkpoint)
    }

    pub fn remove_file_upload_checkpoint(
        &self,
        path: &VaultPath,
    ) -> Result<Option<UploadCheckpoint>> {
        database::remove_file_upload_checkpoint(&self.conn, path)
    }

    pub fn insert_pending_delete(&self, kind: ResourceKind, path: &VaultPath) -> Result<()> {
        database::insert_pending_delete(&self.conn, kind, path)
    }

    pub fn remove_pending_delete(&self, kind: ResourceKind, path: &VaultPath) -> Result<bool> {
        database::remove_pending_delete(&self.conn, kind, path)
    }

    pub fn push_pending_rename(&self, kind: ResourceKind, rename: PendingRename) -> Result<()> {
        database::push_pending_rename(&self.conn, kind, &rename)
    }

    pub fn pop_pending_rename(&self, kind: ResourceKind) -> Result<Option<PendingRename>> {
        database::pop_pending_rename(&self.conn, kind)
    }

    pub fn restore_download_chunks(&self, session: &mut DownloadSession) -> Result<()> {
        database::restore_download_chunks(&self.conn, session)
    }

    pub fn save_download_chunk(
        &self,
        session: &DownloadSession,
        chunk_index: u32,
        chunk_data: &[u8],
    ) -> Result<()> {
        database::save_download_chunk(&self.conn, session, chunk_index, chunk_data)
    }

    pub fn clear_download_chunks(
        &self,
        content_hash: &ContentHash,
        size: u64,
        chunk_size: usize,
    ) -> Result<()> {
        database::clear_download_chunks(&self.conn, content_hash, size, chunk_size)
    }
}
