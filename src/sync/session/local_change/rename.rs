use crate::core::{ContentHash, ResourceKind, VaultPath};
use crate::hash::{file_content_hash, path_hash, text_content_hash};
use crate::protocol::{Action, FileRenameRequest, FolderRenameRequest, NoteRenameRequest};
use crate::store::PendingRename;
use tracing::debug;

use crate::sync::session::{Result, local_change::LocalChangeSender};

impl LocalChangeSender<'_> {
    pub(super) async fn send_path_rename(
        &mut self,
        old_path: &VaultPath,
        new_path: &VaultPath,
    ) -> Result<()> {
        let scan_options = self.config.scan_options()?;

        let old_ignored = scan_options.should_ignore(old_path);
        let new_ignored = scan_options.should_ignore(new_path);
        match (old_ignored, new_ignored) {
            (true, true) => return Ok(()),
            (true, false) => {
                return self.send_path_change(new_path).await;
            }
            (false, true) => {
                return self.send_path_change(old_path).await;
            }
            (false, false) => {}
        }

        if self
            .store
            .hash_entry(ResourceKind::Note, old_path)?
            .is_some()
        {
            if self.can_rename_note(old_path, new_path)? {
                return self.send_note_rename(old_path, new_path).await;
            }
        } else if self
            .store
            .hash_entry(ResourceKind::Setting, old_path)?
            .is_some()
        {
            return self.send_setting_rename_as_delete_modify(old_path, new_path).await;
        } else if self
            .store
            .hash_entry(ResourceKind::File, old_path)?
            .is_some()
        {
            if self.can_rename_file(old_path, new_path)? && !scan_options.is_setting_path(new_path)
            {
                return self.send_file_rename(old_path, new_path).await;
            }
        } else if self
            .store
            .hash_entry(ResourceKind::Folder, old_path)?
            .is_some()
            && self.can_rename_folder(old_path, new_path)
        {
            return self.send_folder_rename(old_path, new_path).await;
        }

        self.send_path_change(old_path).await?;
        self.send_path_change(new_path).await
    }

    async fn send_setting_rename_as_delete_modify(
        &mut self,
        old_path: &VaultPath,
        new_path: &VaultPath,
    ) -> Result<()> {
        self.send_path_change(old_path).await?;
        self.send_path_change(new_path).await?;
        debug!(old_path = %old_path, new_path = %new_path, "sent setting rename as delete and modify");
        Ok(())
    }

    async fn send_note_rename(&mut self, old_path: &VaultPath, new_path: &VaultPath) -> Result<()> {
        let content_hash = self.stored_content_hash(ResourceKind::Note, old_path)?;
        let request = NoteRenameRequest {
            vault: self.vault_name.to_string(),
            path: new_path.to_string(),
            path_hash: path_hash(new_path.as_str())?.to_string(),
            old_path: old_path.to_string(),
            old_path_hash: path_hash(old_path.as_str())?.to_string(),
        };

        self.ws.send_json(Action::NoteRename, &request).await?;
        self.move_hash_entry(ResourceKind::Note, old_path, new_path, content_hash)?;
        debug!(old_path = %old_path, new_path = %new_path, "sent note rename from watch event");
        Ok(())
    }

    async fn send_file_rename(&mut self, old_path: &VaultPath, new_path: &VaultPath) -> Result<()> {
        let content_hash = self.stored_content_hash(ResourceKind::File, old_path)?;
        let request = FileRenameRequest {
            vault: self.vault_name.to_string(),
            path: new_path.to_string(),
            path_hash: path_hash(new_path.as_str())?.to_string(),
            old_path: old_path.to_string(),
            old_path_hash: path_hash(old_path.as_str())?.to_string(),
        };

        self.ws.send_json(Action::FileRename, &request).await?;
        self.move_hash_entry(ResourceKind::File, old_path, new_path, content_hash)?;
        debug!(old_path = %old_path, new_path = %new_path, "sent file rename from watch event");
        Ok(())
    }

    async fn send_folder_rename(
        &mut self,
        old_path: &VaultPath,
        new_path: &VaultPath,
    ) -> Result<()> {
        let request = FolderRenameRequest {
            vault: self.vault_name.to_string(),
            path: new_path.to_string(),
            path_hash: path_hash(new_path.as_str())?.to_string(),
            old_path: old_path.to_string(),
            old_path_hash: path_hash(old_path.as_str())?.to_string(),
        };

        self.ws.send_json(Action::FolderRename, &request).await?;
        self.store.rename_hash_tree(old_path, new_path)?;
        self.store.push_pending_rename(
            ResourceKind::Folder,
            PendingRename::new(old_path.clone(), new_path.clone(), None),
        )?;
        debug!(old_path = %old_path, new_path = %new_path, "sent folder rename from watch event");
        Ok(())
    }

    fn move_hash_entry(
        &mut self,
        kind: ResourceKind,
        old_path: &VaultPath,
        new_path: &VaultPath,
        content_hash: Option<ContentHash>,
    ) -> Result<()> {
        let entry = self.store.remove_hash_entry(kind, old_path)?;
        if let Some(entry) = entry {
            self.store.set_hash_entry(kind, new_path, entry)?;
        }
        self.store.push_pending_rename(
            kind,
            PendingRename::new(old_path.clone(), new_path.clone(), content_hash),
        )?;
        Ok(())
    }

    fn can_rename_note(&self, old_path: &VaultPath, new_path: &VaultPath) -> Result<bool> {
        if !new_path.to_path_buf_under(self.vault.root()).is_file()
            || !super::is_note_path(new_path)
        {
            return Ok(false);
        }

        let Some(old_hash) = self.stored_content_hash(ResourceKind::Note, old_path)? else {
            return Ok(true);
        };

        let content = self.vault.read_text(new_path)?;
        Ok(text_content_hash(&content) == old_hash)
    }

    fn can_rename_file(&self, old_path: &VaultPath, new_path: &VaultPath) -> Result<bool> {
        if !new_path.to_path_buf_under(self.vault.root()).is_file() || super::is_note_path(new_path)
        {
            return Ok(false);
        }

        let Some(old_hash) = self.stored_content_hash(ResourceKind::File, old_path)? else {
            return Ok(true);
        };

        let bytes = self.vault.read_bytes(new_path)?;
        Ok(file_content_hash(&bytes) == old_hash)
    }

    fn can_rename_folder(&self, old_path: &VaultPath, new_path: &VaultPath) -> bool {
        let new_absolute = new_path.to_path_buf_under(self.vault.root());
        new_absolute.is_dir() && !old_path.as_str().is_empty()
    }

    fn stored_content_hash(
        &self,
        kind: ResourceKind,
        path: &VaultPath,
    ) -> Result<Option<ContentHash>> {
        let Some(entry) = self.store.hash_entry(kind, path)? else {
            return Ok(None);
        };
        Ok(entry.content_hash()?)
    }
}
