use crate::core::{ResourceKind, VaultPath};
use crate::hash::path_hash;
use crate::protocol::{
    Action, FileDeleteRequest, FolderDeleteRequest, NoteDeleteRequest, SettingDeleteRequest,
};
use tracing::debug;

use crate::sync::session::{Result, local_change::LocalChangeSender};

impl LocalChangeSender<'_> {
    pub(super) async fn send_delete_by_known_kind(&mut self, path: &VaultPath) -> Result<()> {
        if self.store.hash_entry(ResourceKind::Note, path)?.is_some() {
            let request = NoteDeleteRequest {
                vault: self.vault_name.to_string(),
                path: path.to_string(),
                path_hash: path_hash(path.as_str())?.to_string(),
            };
            self.ws.send_json(Action::NoteDelete, &request).await?;
            self.store.insert_pending_delete(ResourceKind::Note, path)?;
        } else if self.store.hash_entry(ResourceKind::File, path)?.is_some() {
            let request = FileDeleteRequest {
                vault: self.vault_name.to_string(),
                path: path.to_string(),
                path_hash: path_hash(path.as_str())?.to_string(),
            };
            self.ws.send_json(Action::FileDelete, &request).await?;
            self.store.insert_pending_delete(ResourceKind::File, path)?;
        } else if self
            .store
            .hash_entry(ResourceKind::Setting, path)?
            .is_some()
        {
            let request = SettingDeleteRequest {
                vault: self.vault_name.to_string(),
                path: path.to_string(),
                path_hash: path_hash(path.as_str())?.to_string(),
            };
            self.ws.send_json(Action::SettingDelete, &request).await?;
            self.store
                .insert_pending_delete(ResourceKind::Setting, path)?;
        } else if self.store.hash_entry(ResourceKind::Folder, path)?.is_some() {
            let request = FolderDeleteRequest {
                vault: self.vault_name.to_string(),
                path: path.to_string(),
                path_hash: path_hash(path.as_str())?.to_string(),
            };
            self.ws.send_json(Action::FolderDelete, &request).await?;
            self.store
                .insert_pending_delete(ResourceKind::Folder, path)?;
        }

        debug!(path = %path, "sent delete from watch event");
        Ok(())
    }

    pub(super) fn is_deleted_path_unchanged(&self, path: &VaultPath) -> Result<bool> {
        for kind in [
            ResourceKind::Note,
            ResourceKind::File,
            ResourceKind::Folder,
            ResourceKind::Setting,
        ] {
            if self.store.hash_entry(kind, path)?.is_some() {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
