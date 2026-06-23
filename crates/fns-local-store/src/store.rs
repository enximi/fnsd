use std::path::{Path, PathBuf};

use fns_core::{ContentHash, RemoteMillis, ResourceKind, VaultPath};

use crate::{HashEntry, LocalStoreState, PendingRename, Result, UploadCheckpoint, error::io};

#[derive(Debug, Clone)]
pub struct LocalStore {
    path: PathBuf,
    state: LocalStoreState,
}

impl LocalStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();

        if !path.exists() {
            return Ok(Self {
                path,
                state: LocalStoreState::default(),
            });
        }

        let content = std::fs::read_to_string(&path).map_err(|err| io(&path, err))?;
        let state = serde_json::from_str(&content)?;

        Ok(Self { path, state })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn state(&self) -> &LocalStoreState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut LocalStoreState {
        &mut self.state
    }

    pub fn sync_time(&self, kind: ResourceKind) -> Result<RemoteMillis> {
        self.state.sync_times.get(kind)
    }

    pub fn set_sync_time(&mut self, kind: ResourceKind, value: RemoteMillis) {
        self.state.sync_times.set(kind, value);
    }

    pub fn hash_entry(&self, kind: ResourceKind, path: &VaultPath) -> Option<&HashEntry> {
        self.state.hashes.by_kind(kind).get(path.as_str())
    }

    pub fn set_hash_entry(&mut self, kind: ResourceKind, path: &VaultPath, entry: HashEntry) {
        self.state
            .hashes
            .by_kind_mut(kind)
            .insert(path.to_string(), entry);
    }

    pub fn set_content_hash(
        &mut self,
        kind: ResourceKind,
        path: &VaultPath,
        content_hash: Option<ContentHash>,
        mtime: RemoteMillis,
        size: u64,
    ) {
        self.set_hash_entry(kind, path, HashEntry::new(content_hash, mtime, size));
    }

    pub fn remove_hash_entry(&mut self, kind: ResourceKind, path: &VaultPath) -> Option<HashEntry> {
        self.state.hashes.by_kind_mut(kind).remove(path.as_str())
    }

    pub fn all_hash_paths(&self, kind: ResourceKind) -> Result<Vec<VaultPath>> {
        self.state
            .hashes
            .by_kind(kind)
            .keys()
            .map(VaultPath::new)
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    pub fn clear_hashes(&mut self, kind: ResourceKind) {
        self.state.hashes.by_kind_mut(kind).clear();
    }

    pub fn set_pending_modify(
        &mut self,
        kind: ResourceKind,
        path: &VaultPath,
        content_hash: &ContentHash,
    ) {
        match kind {
            ResourceKind::Note => {
                self.state
                    .pending
                    .note_modifies
                    .insert(path.to_string(), content_hash.to_string());
            }
            ResourceKind::File => {
                self.state
                    .pending
                    .file_uploads
                    .insert(path.to_string(), content_hash.to_string());
            }
            ResourceKind::Setting => {
                self.state
                    .pending
                    .setting_modifies
                    .insert(path.to_string(), content_hash.to_string());
            }
            ResourceKind::Folder => {}
        }
    }

    pub fn remove_pending_modify(
        &mut self,
        kind: ResourceKind,
        path: &VaultPath,
    ) -> Option<String> {
        match kind {
            ResourceKind::Note => self.state.pending.note_modifies.remove(path.as_str()),
            ResourceKind::File => self.state.pending.file_uploads.remove(path.as_str()),
            ResourceKind::Setting => self.state.pending.setting_modifies.remove(path.as_str()),
            ResourceKind::Folder => None,
        }
    }

    pub fn has_pending_modify(&self, kind: ResourceKind, path: &VaultPath) -> bool {
        match kind {
            ResourceKind::Note => self.state.pending.note_modifies.contains_key(path.as_str()),
            ResourceKind::File => self.state.pending.file_uploads.contains_key(path.as_str()),
            ResourceKind::Setting => self
                .state
                .pending
                .setting_modifies
                .contains_key(path.as_str()),
            ResourceKind::Folder => false,
        }
    }

    pub fn file_upload_checkpoint(&self, path: &VaultPath) -> Option<&UploadCheckpoint> {
        self.state
            .pending
            .file_upload_checkpoints
            .get(path.as_str())
    }

    pub fn set_file_upload_checkpoint(&mut self, path: &VaultPath, checkpoint: UploadCheckpoint) {
        self.state
            .pending
            .file_upload_checkpoints
            .insert(path.to_string(), checkpoint);
    }

    pub fn remove_file_upload_checkpoint(&mut self, path: &VaultPath) -> Option<UploadCheckpoint> {
        self.state
            .pending
            .file_upload_checkpoints
            .remove(path.as_str())
    }

    pub fn insert_pending_delete(&mut self, kind: ResourceKind, path: &VaultPath) {
        match kind {
            ResourceKind::Note => {
                self.state.pending.note_deletes.insert(path.to_string());
            }
            ResourceKind::File => {
                self.state.pending.file_deletes.insert(path.to_string());
            }
            ResourceKind::Folder => {
                self.state.pending.folder_deletes.insert(path.to_string());
            }
            ResourceKind::Setting => {
                self.state.pending.setting_deletes.insert(path.to_string());
            }
        }
    }

    pub fn remove_pending_delete(&mut self, kind: ResourceKind, path: &VaultPath) -> bool {
        match kind {
            ResourceKind::Note => self.state.pending.note_deletes.remove(path.as_str()),
            ResourceKind::File => self.state.pending.file_deletes.remove(path.as_str()),
            ResourceKind::Folder => self.state.pending.folder_deletes.remove(path.as_str()),
            ResourceKind::Setting => self.state.pending.setting_deletes.remove(path.as_str()),
        }
    }

    pub fn push_pending_rename(&mut self, kind: ResourceKind, rename: PendingRename) {
        match kind {
            ResourceKind::Note => self.state.pending.note_renames.push(rename),
            ResourceKind::File => self.state.pending.file_renames.push(rename),
            ResourceKind::Folder => self.state.pending.folder_renames.push(rename),
            ResourceKind::Setting => {}
        }
    }

    pub fn pop_pending_rename(&mut self, kind: ResourceKind) -> Option<PendingRename> {
        match kind {
            ResourceKind::Note => pop_front(&mut self.state.pending.note_renames),
            ResourceKind::File => pop_front(&mut self.state.pending.file_renames),
            ResourceKind::Folder => pop_front(&mut self.state.pending.folder_renames),
            ResourceKind::Setting => None,
        }
    }

    pub fn save(&self) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).map_err(|err| io(parent, err))?;
        }

        let content = serde_json::to_string_pretty(&self.state)?;
        std::fs::write(&self.path, content).map_err(|err| io(&self.path, err))?;
        Ok(())
    }
}

fn pop_front<T>(items: &mut Vec<T>) -> Option<T> {
    if items.is_empty() {
        None
    } else {
        Some(items.remove(0))
    }
}
