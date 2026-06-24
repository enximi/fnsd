use crate::core::{
    ContentHash, DeletedResource, FileResource, FolderResource, RemoteMillis, SyncBatch,
    TextResource, VaultName, VaultPath,
};
use crate::protocol::{
    FileSyncCheckRequest, FileSyncDelFile, FileSyncRequest, FolderSyncCheckRequest,
    FolderSyncDelFolder, FolderSyncRequest, NoteModifyOrCreateRequest, NoteSyncCheckRequest,
    NoteSyncDelNote, NoteSyncRequest, SettingModifyOrCreateRequest, SettingSyncCheckRequest,
    SettingSyncDelSetting, SettingSyncRequest,
};

use crate::sync::plan::{Result, error::PlanError};

pub fn build_note_sync_request(
    vault: &VaultName,
    batch: &SyncBatch<TextResource>,
) -> NoteSyncRequest {
    NoteSyncRequest {
        context: batch.context.clone(),
        vault: vault.to_string(),
        last_time: batch.last_time.as_i64(),
        notes: batch.items.iter().map(note_check).collect(),
        del_notes: batch.deleted.iter().map(note_deleted).collect(),
        missing_notes: batch.missing.iter().map(note_deleted).collect(),
    }
}

pub fn build_file_sync_request(
    vault: &VaultName,
    batch: &SyncBatch<FileResource>,
) -> Result<FileSyncRequest> {
    Ok(FileSyncRequest {
        context: batch.context.clone(),
        vault: vault.to_string(),
        last_time: batch.last_time.as_i64(),
        files: batch
            .items
            .iter()
            .map(file_check)
            .collect::<Result<Vec<_>>>()?,
        del_files: batch.deleted.iter().map(file_deleted).collect(),
        missing_files: batch.missing.iter().map(file_deleted).collect(),
    })
}

pub fn build_folder_sync_request(
    vault: &VaultName,
    batch: &SyncBatch<FolderResource>,
) -> FolderSyncRequest {
    FolderSyncRequest {
        context: batch.context.clone(),
        vault: vault.to_string(),
        last_time: batch.last_time.as_i64(),
        folders: batch.items.iter().map(folder_check).collect(),
        del_folders: batch.deleted.iter().map(folder_deleted).collect(),
        missing_folders: batch.missing.iter().map(folder_deleted).collect(),
    }
}

pub fn build_setting_sync_request(
    vault: &VaultName,
    batch: &SyncBatch<TextResource>,
) -> SettingSyncRequest {
    SettingSyncRequest {
        context: batch.context.clone(),
        vault: vault.to_string(),
        last_time: batch.last_time.as_i64(),
        settings: batch.items.iter().map(setting_check).collect(),
        del_settings: batch.deleted.iter().map(setting_deleted).collect(),
        missing_settings: batch.missing.iter().map(setting_deleted).collect(),
    }
}

pub fn build_note_modify_request(
    vault: &VaultName,
    path: &VaultPath,
    content: String,
    content_hash: &ContentHash,
    ctime: RemoteMillis,
    mtime: RemoteMillis,
) -> Result<NoteModifyOrCreateRequest> {
    Ok(NoteModifyOrCreateRequest {
        vault: vault.to_string(),
        path: path.to_string(),
        path_hash: crate::hash::path_hash(path.as_str())?.to_string(),
        base_hash: None,
        base_hash_missing: false,
        content,
        content_hash: content_hash.to_string(),
        ctime: ctime.as_i64(),
        mtime: mtime.as_i64(),
        create_only: false,
    })
}

pub fn build_setting_modify_request(
    vault: &VaultName,
    path: &VaultPath,
    content: String,
    content_hash: &ContentHash,
    ctime: RemoteMillis,
    mtime: RemoteMillis,
) -> Result<SettingModifyOrCreateRequest> {
    Ok(SettingModifyOrCreateRequest {
        vault: vault.to_string(),
        path: path.to_string(),
        path_hash: crate::hash::path_hash(path.as_str())?.to_string(),
        content,
        content_hash: content_hash.to_string(),
        ctime: ctime.as_i64(),
        mtime: mtime.as_i64(),
    })
}

fn note_check(item: &TextResource) -> NoteSyncCheckRequest {
    NoteSyncCheckRequest {
        path: item.path.to_string(),
        path_hash: item.path_hash.to_string(),
        content_hash: item.content_hash.to_string(),
        mtime: item.mtime.as_i64(),
        ctime: item.ctime.as_i64(),
    }
}

fn note_deleted(item: &DeletedResource) -> NoteSyncDelNote {
    NoteSyncDelNote {
        path: item.path.to_string(),
        path_hash: item.path_hash.to_string(),
    }
}

fn file_check(item: &FileResource) -> Result<FileSyncCheckRequest> {
    Ok(FileSyncCheckRequest {
        path: item.path.to_string(),
        path_hash: item.path_hash.to_string(),
        content_hash: item.content_hash.to_string(),
        size: size_as_i64(&item.path, item.size)?,
        mtime: item.mtime.as_i64(),
        ctime: item.ctime.as_i64(),
    })
}

fn file_deleted(item: &DeletedResource) -> FileSyncDelFile {
    FileSyncDelFile {
        path: item.path.to_string(),
        path_hash: item.path_hash.to_string(),
    }
}

fn folder_check(item: &FolderResource) -> FolderSyncCheckRequest {
    FolderSyncCheckRequest {
        path: item.path.to_string(),
        path_hash: item.path_hash.to_string(),
        mtime: item.mtime.as_i64(),
    }
}

fn folder_deleted(item: &DeletedResource) -> FolderSyncDelFolder {
    FolderSyncDelFolder {
        path: item.path.to_string(),
        path_hash: item.path_hash.to_string(),
    }
}

fn setting_check(item: &TextResource) -> SettingSyncCheckRequest {
    SettingSyncCheckRequest {
        path: item.path.to_string(),
        path_hash: item.path_hash.to_string(),
        content_hash: item.content_hash.to_string(),
        mtime: item.mtime.as_i64(),
        ctime: item.ctime.as_i64(),
    }
}

fn setting_deleted(item: &DeletedResource) -> SettingSyncDelSetting {
    SettingSyncDelSetting {
        path: item.path.to_string(),
        path_hash: item.path_hash.to_string(),
    }
}

fn size_as_i64(path: &VaultPath, size: u64) -> Result<i64> {
    i64::try_from(size).map_err(|_| PlanError::FileSizeTooLarge {
        path: path.clone(),
        size,
    })
}
