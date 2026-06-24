use crate::core::{RemoteMillis, ResourceKind, VaultPath};
use crate::hash::{file_content_hash, path_hash, text_content_hash};
use crate::protocol::{Action, FileUploadCheckRequest, FolderCreateRequest};
use crate::sync::plan::{build_note_modify_request, build_setting_modify_request};
use crate::vault::fs::VaultScanOptions;
use tracing::debug;

use crate::sync::session::{Result, local_change::LocalChangeSender};

impl LocalChangeSender<'_> {
    pub(super) async fn send_path_change(&mut self, path: &VaultPath) -> Result<()> {
        let scan_options = self.config.scan_options()?;

        if scan_options.should_ignore(path) {
            return Ok(());
        }

        let absolute = path.to_path_buf_under(self.vault.root());
        let metadata = std::fs::symlink_metadata(&absolute).ok();

        match metadata {
            Some(metadata) if metadata.file_type().is_dir() => {
                if self.is_current_path_unchanged(&scan_options, path, &metadata)? {
                    return Ok(());
                }
                self.send_folder_modify(path, &metadata).await?;
            }
            Some(metadata) if metadata.file_type().is_file() => {
                if self.is_current_path_unchanged(&scan_options, path, &metadata)? {
                    return Ok(());
                }
                if scan_options.is_setting_path(path) {
                    self.send_setting_modify(path).await?;
                } else if super::is_note_path(path) {
                    self.send_note_modify(path).await?;
                } else {
                    self.send_file_upload_check(path).await?;
                }
            }
            Some(_) => {}
            None => {
                if self.is_deleted_path_unchanged(path)? {
                    return Ok(());
                }
                self.send_delete_by_known_kind(path).await?;
            }
        }

        Ok(())
    }

    async fn send_note_modify(&mut self, path: &VaultPath) -> Result<()> {
        let content = self.vault.read_text(path)?;
        let metadata = self.vault.file_metadata(path)?;
        let content_hash = text_content_hash(&content);
        let request = build_note_modify_request(
            self.vault_name,
            path,
            content,
            &content_hash,
            metadata.ctime,
            metadata.mtime,
        )?;

        self.ws.send_json(Action::NoteModify, &request).await?;
        self.store
            .set_pending_modify(ResourceKind::Note, path, &content_hash)?;
        self.store.set_content_hash(
            ResourceKind::Note,
            path,
            Some(content_hash),
            metadata.mtime,
            metadata.size,
        )?;
        debug!(path = %path, "sent note modify from watch event");
        Ok(())
    }

    async fn send_setting_modify(&mut self, path: &VaultPath) -> Result<()> {
        let content = self.vault.read_text(path)?;
        let metadata = self.vault.file_metadata(path)?;
        let content_hash = text_content_hash(&content);
        let request = build_setting_modify_request(
            self.vault_name,
            path,
            content,
            &content_hash,
            metadata.ctime,
            metadata.mtime,
        )?;

        self.ws.send_json(Action::SettingModify, &request).await?;
        self.store
            .set_pending_modify(ResourceKind::Setting, path, &content_hash)?;
        self.store.set_content_hash(
            ResourceKind::Setting,
            path,
            Some(content_hash),
            metadata.mtime,
            metadata.size,
        )?;
        debug!(path = %path, "sent setting modify from watch event");
        Ok(())
    }

    async fn send_file_upload_check(&mut self, path: &VaultPath) -> Result<()> {
        let bytes = self.vault.read_bytes(path)?;
        let metadata = self.vault.file_metadata(path)?;
        let content_hash = file_content_hash(&bytes);
        let request = FileUploadCheckRequest {
            vault: self.vault_name.to_string(),
            path: path.to_string(),
            path_hash: path_hash(path.as_str())?.to_string(),
            content_hash: content_hash.to_string(),
            size: i64::try_from(metadata.size).unwrap_or(i64::MAX),
            ctime: metadata.ctime.as_i64(),
            mtime: metadata.mtime.as_i64(),
        };

        self.ws.send_json(Action::FileUploadCheck, &request).await?;
        self.store.set_content_hash(
            ResourceKind::File,
            path,
            Some(content_hash),
            metadata.mtime,
            metadata.size,
        )?;
        debug!(path = %path, "sent file upload check from watch event");
        Ok(())
    }

    async fn send_folder_modify(
        &mut self,
        path: &VaultPath,
        metadata: &std::fs::Metadata,
    ) -> Result<()> {
        let request = FolderCreateRequest {
            vault: self.vault_name.to_string(),
            path: path.to_string(),
            path_hash: path_hash(path.as_str())?.to_string(),
        };

        self.ws.send_json(Action::FolderModify, &request).await?;
        self.store.set_content_hash(
            ResourceKind::Folder,
            path,
            None,
            modified_millis(metadata),
            0,
        )?;
        debug!(path = %path, "sent folder modify from watch event");
        Ok(())
    }

    fn is_current_path_unchanged(
        &self,
        scan_options: &VaultScanOptions,
        path: &VaultPath,
        metadata: &std::fs::Metadata,
    ) -> Result<bool> {
        if metadata.file_type().is_dir() {
            return Ok(self.store.hash_entry(ResourceKind::Folder, path)?.is_some());
        }

        if scan_options.is_setting_path(path) {
            return self.text_file_unchanged(ResourceKind::Setting, path);
        }

        if super::is_note_path(path) {
            return self.text_file_unchanged(ResourceKind::Note, path);
        }

        self.binary_file_unchanged(path)
    }

    fn text_file_unchanged(&self, kind: ResourceKind, path: &VaultPath) -> Result<bool> {
        let Some(entry) = self.store.hash_entry(kind, path)? else {
            return Ok(false);
        };
        let content = self.vault.read_text(path)?;
        Ok(entry.content_hash()?.as_ref() == Some(&text_content_hash(&content)))
    }

    fn binary_file_unchanged(&self, path: &VaultPath) -> Result<bool> {
        let Some(entry) = self.store.hash_entry(ResourceKind::File, path)? else {
            return Ok(false);
        };
        let bytes = self.vault.read_bytes(path)?;
        Ok(
            entry.content_hash()?.as_ref() == Some(&file_content_hash(&bytes))
                && entry.size == bytes.len() as u64,
        )
    }
}

fn modified_millis(metadata: &std::fs::Metadata) -> RemoteMillis {
    metadata
        .modified()
        .ok()
        .and_then(system_time_millis)
        .unwrap_or_else(|| RemoteMillis::new(0).expect("zero timestamp is valid"))
}

fn system_time_millis(time: std::time::SystemTime) -> Option<RemoteMillis> {
    let duration = time.duration_since(std::time::UNIX_EPOCH).ok()?;
    let millis = i64::try_from(duration.as_millis()).ok()?;
    RemoteMillis::new(millis).ok()
}
