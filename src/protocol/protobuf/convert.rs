use crate::protocol::{
    ClientInfoMessage, FileChunkFrame, FileDeleteRequest, FileGetRequest, FileRenameRequest,
    FileSyncRequest, FileUploadCheckRequest, FolderCreateRequest, FolderDeleteRequest,
    FolderRenameRequest, FolderSyncRequest, NoteDeleteRequest, NoteGetRequest,
    NoteModifyOrCreateRequest, NoteRenameRequest, NoteSyncRequest, NoteUpdateCheckRequest,
    ProtocolError, Result, SettingClearRequest, SettingDeleteRequest, SettingGetRequest,
    SettingModifyOrCreateRequest, SettingSyncRequest, SettingUpdateCheckRequest,
};

use super::model::*;

impl From<&ClientInfoMessage> for PbClientInfoMessage {
    fn from(value: &ClientInfoMessage) -> Self {
        Self {
            name: value.name.clone(),
            version: value.version.clone(),
            client_type: value.client_type.clone(),
            is_desktop: value.is_desktop,
            is_mobile: value.is_mobile,
            is_phone: value.is_phone,
            is_tablet: value.is_tablet,
            is_mac_os: value.is_mac_os,
            is_win: value.is_win,
            is_linux: value.is_linux,
            offline_sync_strategy: value
                .offline_sync_strategy
                .map(|strategy| serde_json::to_value(strategy).unwrap_or_default())
                .and_then(|value| value.as_str().map(str::to_string))
                .unwrap_or_default(),
            protobuf: value.protobuf,
        }
    }
}

impl From<ClientInfoMessage> for PbClientInfoMessage {
    fn from(value: ClientInfoMessage) -> Self {
        Self::from(&value)
    }
}

impl From<NoteSyncRequest> for PbNoteSyncRequest {
    fn from(value: NoteSyncRequest) -> Self {
        Self {
            context: value.context.unwrap_or_default(),
            vault: value.vault,
            last_time: value.last_time,
            notes: value.notes.into_iter().map(Into::into).collect(),
            del_notes: value.del_notes.into_iter().map(Into::into).collect(),
            missing_notes: value.missing_notes.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<crate::protocol::NoteSyncCheckRequest> for NoteSyncCheckRequest {
    fn from(value: crate::protocol::NoteSyncCheckRequest) -> Self {
        Self {
            path: value.path,
            path_hash: value.path_hash,
            content_hash: value.content_hash,
            mtime: value.mtime,
            ctime: value.ctime,
        }
    }
}

impl From<crate::protocol::NoteSyncDelNote> for NoteSyncDelNote {
    fn from(value: crate::protocol::NoteSyncDelNote) -> Self {
        Self {
            path: value.path,
            path_hash: value.path_hash,
        }
    }
}

impl From<NoteModifyOrCreateRequest> for PbNoteModifyOrCreateRequest {
    fn from(value: NoteModifyOrCreateRequest) -> Self {
        Self {
            vault: value.vault,
            path: value.path,
            path_hash: value.path_hash,
            base_hash: value.base_hash.unwrap_or_default(),
            base_hash_missing: value.base_hash_missing,
            content: value.content,
            content_hash: value.content_hash,
            ctime: value.ctime,
            mtime: value.mtime,
            create_only: value.create_only,
        }
    }
}

impl From<NoteUpdateCheckRequest> for PbNoteUpdateCheckRequest {
    fn from(value: NoteUpdateCheckRequest) -> Self {
        Self {
            vault: value.vault,
            path: value.path,
            path_hash: value.path_hash,
            content_hash: value.content_hash,
            ctime: value.ctime,
            mtime: value.mtime,
        }
    }
}

impl From<NoteDeleteRequest> for PbNoteDeleteRequest {
    fn from(value: NoteDeleteRequest) -> Self {
        Self {
            vault: value.vault,
            path: value.path,
            path_hash: value.path_hash,
        }
    }
}

impl From<NoteRenameRequest> for PbNoteRenameRequest {
    fn from(value: NoteRenameRequest) -> Self {
        Self {
            vault: value.vault,
            path: value.path,
            path_hash: value.path_hash,
            old_path: value.old_path,
            old_path_hash: value.old_path_hash,
        }
    }
}

impl From<NoteGetRequest> for PbNoteGetRequest {
    fn from(value: NoteGetRequest) -> Self {
        Self {
            vault: value.vault,
            path: value.path,
            path_hash: value.path_hash,
            is_recycle: value.is_recycle,
        }
    }
}

impl From<FileSyncRequest> for PbFileSyncRequest {
    fn from(value: FileSyncRequest) -> Self {
        Self {
            context: value.context.unwrap_or_default(),
            vault: value.vault,
            last_time: value.last_time,
            files: value.files.into_iter().map(Into::into).collect(),
            del_files: value.del_files.into_iter().map(Into::into).collect(),
            missing_files: value.missing_files.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<crate::protocol::FileSyncCheckRequest> for FileSyncCheckRequest {
    fn from(value: crate::protocol::FileSyncCheckRequest) -> Self {
        Self {
            path: value.path,
            path_hash: value.path_hash,
            content_hash: value.content_hash,
            size: value.size,
            mtime: value.mtime,
            ctime: value.ctime,
        }
    }
}

impl From<crate::protocol::FileSyncDelFile> for FileSyncDelFile {
    fn from(value: crate::protocol::FileSyncDelFile) -> Self {
        Self {
            path: value.path,
            path_hash: value.path_hash,
        }
    }
}

impl From<FileUploadCheckRequest> for PbFileUploadCheckRequest {
    fn from(value: FileUploadCheckRequest) -> Self {
        Self {
            vault: value.vault,
            path: value.path,
            path_hash: value.path_hash,
            content_hash: value.content_hash,
            size: value.size,
            ctime: value.ctime,
            mtime: value.mtime,
        }
    }
}

impl From<FileDeleteRequest> for PbFileDeleteRequest {
    fn from(value: FileDeleteRequest) -> Self {
        Self {
            vault: value.vault,
            path: value.path,
            path_hash: value.path_hash,
        }
    }
}

impl From<FileRenameRequest> for PbFileRenameRequest {
    fn from(value: FileRenameRequest) -> Self {
        Self {
            vault: value.vault,
            path: value.path,
            path_hash: value.path_hash,
            old_path: value.old_path,
            old_path_hash: value.old_path_hash,
        }
    }
}

impl From<FileGetRequest> for PbFileChunkDownloadRequest {
    fn from(value: FileGetRequest) -> Self {
        Self {
            vault: value.vault,
            path: value.path,
            path_hash: value.path_hash,
            session_id: String::new(),
            chunk_index: 0,
        }
    }
}

impl From<FileGetRequest> for PbFileGetRequest {
    fn from(value: FileGetRequest) -> Self {
        Self {
            vault: value.vault,
            path: value.path,
            path_hash: value.path_hash,
        }
    }
}

impl From<SettingSyncRequest> for PbSettingSyncRequest {
    fn from(value: SettingSyncRequest) -> Self {
        Self {
            context: value.context.unwrap_or_default(),
            vault: value.vault,
            last_time: value.last_time,
            settings: value.settings.into_iter().map(Into::into).collect(),
            del_settings: value.del_settings.into_iter().map(Into::into).collect(),
            missing_settings: value.missing_settings.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<crate::protocol::SettingSyncCheckRequest> for SettingSyncCheckRequest {
    fn from(value: crate::protocol::SettingSyncCheckRequest) -> Self {
        Self {
            path: value.path,
            path_hash: value.path_hash,
            content_hash: value.content_hash,
            mtime: value.mtime,
            ctime: value.ctime,
        }
    }
}

impl From<crate::protocol::SettingSyncDelSetting> for SettingSyncDelSetting {
    fn from(value: crate::protocol::SettingSyncDelSetting) -> Self {
        Self {
            path: value.path,
            path_hash: value.path_hash,
        }
    }
}

impl From<SettingModifyOrCreateRequest> for PbSettingModifyOrCreateRequest {
    fn from(value: SettingModifyOrCreateRequest) -> Self {
        Self {
            vault: value.vault,
            path: value.path,
            path_hash: value.path_hash,
            content: value.content,
            content_hash: value.content_hash,
            ctime: value.ctime,
            mtime: value.mtime,
        }
    }
}

impl From<SettingUpdateCheckRequest> for PbSettingUpdateCheckRequest {
    fn from(value: SettingUpdateCheckRequest) -> Self {
        Self {
            vault: value.vault,
            path: value.path,
            path_hash: value.path_hash,
            content_hash: value.content_hash,
            ctime: value.ctime,
            mtime: value.mtime,
        }
    }
}

impl From<SettingDeleteRequest> for PbSettingDeleteRequest {
    fn from(value: SettingDeleteRequest) -> Self {
        Self {
            vault: value.vault,
            path: value.path,
            path_hash: value.path_hash,
        }
    }
}

impl From<SettingGetRequest> for PbSettingGetRequest {
    fn from(value: SettingGetRequest) -> Self {
        Self {
            vault: value.vault,
            path: value.path,
            path_hash: value.path_hash,
        }
    }
}

impl From<SettingClearRequest> for PbSettingClearRequest {
    fn from(value: SettingClearRequest) -> Self {
        Self { vault: value.vault }
    }
}

impl From<FolderSyncRequest> for PbFolderSyncRequest {
    fn from(value: FolderSyncRequest) -> Self {
        Self {
            context: value.context.unwrap_or_default(),
            vault: value.vault,
            last_time: value.last_time,
            folders: value.folders.into_iter().map(Into::into).collect(),
            del_folders: value.del_folders.into_iter().map(Into::into).collect(),
            missing_folders: value.missing_folders.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<crate::protocol::FolderSyncCheckRequest> for FolderSyncCheckRequest {
    fn from(value: crate::protocol::FolderSyncCheckRequest) -> Self {
        Self {
            path: value.path,
            path_hash: value.path_hash,
            mtime: value.mtime,
        }
    }
}

impl From<crate::protocol::FolderSyncDelFolder> for FolderSyncDelFolder {
    fn from(value: crate::protocol::FolderSyncDelFolder) -> Self {
        Self {
            path: value.path,
            path_hash: value.path_hash,
        }
    }
}

impl From<FolderCreateRequest> for PbFolderCreateRequest {
    fn from(value: FolderCreateRequest) -> Self {
        Self {
            vault: value.vault,
            path: value.path,
            path_hash: value.path_hash,
        }
    }
}

impl From<FolderDeleteRequest> for PbFolderDeleteRequest {
    fn from(value: FolderDeleteRequest) -> Self {
        Self {
            vault: value.vault,
            path: value.path,
            path_hash: value.path_hash,
        }
    }
}

impl From<FolderRenameRequest> for PbFolderRenameRequest {
    fn from(value: FolderRenameRequest) -> Self {
        Self {
            vault: value.vault,
            path: value.path,
            path_hash: value.path_hash,
            old_path: value.old_path,
            old_path_hash: value.old_path_hash,
        }
    }
}

impl TryFrom<FileChunkFrame> for PbFileChunkDownloadRequest {
    type Error = ProtocolError;

    fn try_from(value: FileChunkFrame) -> Result<Self> {
        Ok(Self {
            vault: String::new(),
            path: String::new(),
            path_hash: String::new(),
            session_id: value.session_id().to_string(),
            chunk_index: i64::from(value.chunk_index()),
        })
    }
}
