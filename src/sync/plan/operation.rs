use crate::core::{ContentHash, PathHash, RemoteMillis, VaultPath};
use crate::protocol::{
    FileSyncDeleteMessage, FileSyncDownloadMessage, FileSyncModifyMessage, FileSyncMtimeMessage,
    FileSyncRenameMessage, FileSyncUploadMessage, FolderSyncDeleteMessage, FolderSyncModifyMessage,
    FolderSyncRenameMessage, NoteSyncDeleteMessage, NoteSyncModifyMessage, NoteSyncMtimeMessage,
    NoteSyncNeedPushMessage, NoteSyncRenameMessage, SettingSyncDeleteMessage,
    SettingSyncModifyMessage, SettingSyncMtimeMessage, SettingSyncNeedUploadMessage,
};

use crate::sync::plan::{DeletedResource, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NoteOperation {
    Write(RemoteText),
    Delete(DeletedResource),
    Rename(RemoteTextRename),
    UpdateMtime(MtimeUpdate),
    Upload(DeletedResource),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileOperation {
    Download(RemoteFile),
    Delete(DeletedResource),
    Rename(RemotePathRename),
    UpdateMtime(MtimeUpdate),
    Upload(FileUpload),
    ReceiveDownload(FileDownload),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FolderOperation {
    Create(RemoteFolder),
    Delete(DeletedResource),
    Rename(RemotePathRename),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingOperation {
    Write(RemoteText),
    Delete(DeletedResource),
    UpdateMtime(MtimeUpdate),
    Upload(VaultPath),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteText {
    pub path: VaultPath,
    pub path_hash: PathHash,
    pub content: String,
    pub content_hash: ContentHash,
    pub ctime: RemoteMillis,
    pub mtime: RemoteMillis,
    pub last_time: RemoteMillis,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteFile {
    pub path: VaultPath,
    pub path_hash: PathHash,
    pub content_hash: ContentHash,
    pub size: i64,
    pub ctime: RemoteMillis,
    pub mtime: RemoteMillis,
    pub last_time: RemoteMillis,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteFolder {
    pub path: VaultPath,
    pub path_hash: PathHash,
    pub ctime: RemoteMillis,
    pub mtime: RemoteMillis,
    pub last_time: RemoteMillis,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteTextRename {
    pub path: VaultPath,
    pub path_hash: PathHash,
    pub old_path: VaultPath,
    pub old_path_hash: PathHash,
    pub content_hash: ContentHash,
    pub ctime: RemoteMillis,
    pub mtime: RemoteMillis,
    pub last_time: RemoteMillis,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemotePathRename {
    pub path: VaultPath,
    pub path_hash: PathHash,
    pub old_path: VaultPath,
    pub old_path_hash: PathHash,
    pub ctime: RemoteMillis,
    pub mtime: RemoteMillis,
    pub last_time: RemoteMillis,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MtimeUpdate {
    pub path: VaultPath,
    pub ctime: RemoteMillis,
    pub mtime: RemoteMillis,
    pub last_time: RemoteMillis,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileUpload {
    pub path: VaultPath,
    pub path_hash: PathHash,
    pub session_id: String,
    pub chunk_size: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileDownload {
    pub path: VaultPath,
    pub content_hash: ContentHash,
    pub ctime: RemoteMillis,
    pub mtime: RemoteMillis,
    pub session_id: String,
    pub chunk_size: i64,
    pub total_chunks: i64,
    pub size: i64,
}

pub fn plan_note_modify(message: &NoteSyncModifyMessage) -> Result<NoteOperation> {
    Ok(NoteOperation::Write(RemoteText {
        path: parse_path(&message.path)?,
        path_hash: parse_path_hash(&message.path_hash)?,
        content: message.content.clone(),
        content_hash: parse_content_hash(&message.content_hash)?,
        ctime: parse_time(message.ctime)?,
        mtime: parse_time(message.mtime)?,
        last_time: parse_time(message.last_time)?,
    }))
}

pub fn plan_note_delete(message: &NoteSyncDeleteMessage) -> Result<NoteOperation> {
    Ok(NoteOperation::Delete(deleted(
        &message.path,
        &message.path_hash,
    )?))
}

pub fn plan_note_rename(message: &NoteSyncRenameMessage) -> Result<NoteOperation> {
    Ok(NoteOperation::Rename(RemoteTextRename {
        path: parse_path(&message.path)?,
        path_hash: parse_path_hash(&message.path_hash)?,
        old_path: parse_path(&message.old_path)?,
        old_path_hash: parse_path_hash(&message.old_path_hash)?,
        content_hash: parse_content_hash(&message.content_hash)?,
        ctime: parse_time(message.ctime)?,
        mtime: parse_time(message.mtime)?,
        last_time: parse_time(message.last_time)?,
    }))
}

pub fn plan_note_mtime(message: &NoteSyncMtimeMessage) -> Result<NoteOperation> {
    Ok(NoteOperation::UpdateMtime(mtime_update(
        &message.path,
        message.ctime,
        message.mtime,
        message.last_time,
    )?))
}

pub fn plan_note_need_push(message: &NoteSyncNeedPushMessage) -> Result<NoteOperation> {
    Ok(NoteOperation::Upload(deleted(
        &message.path,
        &message.path_hash,
    )?))
}

pub fn plan_file_modify(message: &FileSyncModifyMessage) -> Result<FileOperation> {
    Ok(FileOperation::Download(RemoteFile {
        path: parse_path(&message.path)?,
        path_hash: parse_path_hash(&message.path_hash)?,
        content_hash: parse_content_hash(&message.content_hash)?,
        size: message.size,
        ctime: parse_time(message.ctime)?,
        mtime: parse_time(message.mtime)?,
        last_time: parse_time(message.last_time)?,
    }))
}

pub fn plan_file_delete(message: &FileSyncDeleteMessage) -> Result<FileOperation> {
    Ok(FileOperation::Delete(deleted(
        &message.path,
        &message.path_hash,
    )?))
}

pub fn plan_file_rename(message: &FileSyncRenameMessage) -> Result<FileOperation> {
    Ok(FileOperation::Rename(RemotePathRename {
        path: parse_path(&message.path)?,
        path_hash: parse_path_hash(&message.path_hash)?,
        old_path: parse_path(&message.old_path)?,
        old_path_hash: parse_path_hash(&message.old_path_hash)?,
        ctime: parse_time(message.ctime)?,
        mtime: parse_time(message.mtime)?,
        last_time: parse_time(message.last_time)?,
    }))
}

pub fn plan_file_mtime(message: &FileSyncMtimeMessage) -> Result<FileOperation> {
    Ok(FileOperation::UpdateMtime(mtime_update(
        &message.path,
        message.ctime,
        message.mtime,
        message.last_time,
    )?))
}

pub fn plan_file_upload(message: &FileSyncUploadMessage) -> Result<FileOperation> {
    Ok(FileOperation::Upload(FileUpload {
        path: parse_path(&message.path)?,
        path_hash: parse_path_hash(&message.path_hash)?,
        session_id: message.session_id.clone(),
        chunk_size: message.chunk_size,
    }))
}

pub fn plan_file_download(message: &FileSyncDownloadMessage) -> Result<FileOperation> {
    Ok(FileOperation::ReceiveDownload(FileDownload {
        path: parse_path(&message.path)?,
        content_hash: parse_content_hash(&message.content_hash)?,
        ctime: parse_time(message.ctime)?,
        mtime: parse_time(message.mtime)?,
        session_id: message.session_id.clone(),
        chunk_size: message.chunk_size,
        total_chunks: message.total_chunks,
        size: message.size,
    }))
}

pub fn plan_folder_modify(message: &FolderSyncModifyMessage) -> Result<FolderOperation> {
    Ok(FolderOperation::Create(RemoteFolder {
        path: parse_path(&message.path)?,
        path_hash: parse_path_hash(&message.path_hash)?,
        ctime: parse_time(message.ctime)?,
        mtime: parse_time(message.mtime)?,
        last_time: parse_time(message.last_time)?,
    }))
}

pub fn plan_folder_delete(message: &FolderSyncDeleteMessage) -> Result<FolderOperation> {
    Ok(FolderOperation::Delete(deleted(
        &message.path,
        &message.path_hash,
    )?))
}

pub fn plan_folder_rename(message: &FolderSyncRenameMessage) -> Result<FolderOperation> {
    Ok(FolderOperation::Rename(RemotePathRename {
        path: parse_path(&message.path)?,
        path_hash: parse_path_hash(&message.path_hash)?,
        old_path: parse_path(&message.old_path)?,
        old_path_hash: parse_path_hash(&message.old_path_hash)?,
        ctime: parse_time(message.ctime)?,
        mtime: parse_time(message.mtime)?,
        last_time: parse_time(message.last_time)?,
    }))
}

pub fn plan_setting_modify(message: &SettingSyncModifyMessage) -> Result<SettingOperation> {
    Ok(SettingOperation::Write(RemoteText {
        path: parse_path(&message.path)?,
        path_hash: parse_path_hash(&message.path_hash)?,
        content: message.content.clone(),
        content_hash: parse_content_hash(&message.content_hash)?,
        ctime: parse_time(message.ctime)?,
        mtime: parse_time(message.mtime)?,
        last_time: parse_time(message.last_time)?,
    }))
}

pub fn plan_setting_delete(message: &SettingSyncDeleteMessage) -> Result<SettingOperation> {
    Ok(SettingOperation::Delete(deleted(
        &message.path,
        &message.path_hash,
    )?))
}

pub fn plan_setting_mtime(message: &SettingSyncMtimeMessage) -> Result<SettingOperation> {
    Ok(SettingOperation::UpdateMtime(mtime_update(
        &message.path,
        message.ctime,
        message.mtime,
        message.last_time,
    )?))
}

pub fn plan_setting_need_upload(
    message: &SettingSyncNeedUploadMessage,
) -> Result<SettingOperation> {
    Ok(SettingOperation::Upload(parse_path(&message.path)?))
}

fn deleted(path: &str, path_hash: &str) -> Result<DeletedResource> {
    Ok(DeletedResource {
        path: parse_path(path)?,
        path_hash: parse_path_hash(path_hash)?,
    })
}

fn mtime_update(path: &str, ctime: i64, mtime: i64, last_time: i64) -> Result<MtimeUpdate> {
    Ok(MtimeUpdate {
        path: parse_path(path)?,
        ctime: parse_time(ctime)?,
        mtime: parse_time(mtime)?,
        last_time: parse_time(last_time)?,
    })
}

fn parse_path(value: &str) -> Result<VaultPath> {
    Ok(VaultPath::new(value)?)
}

fn parse_path_hash(value: &str) -> Result<PathHash> {
    Ok(PathHash::new(value)?)
}

fn parse_content_hash(value: &str) -> Result<ContentHash> {
    Ok(ContentHash::new(value)?)
}

fn parse_time(value: i64) -> Result<RemoteMillis> {
    Ok(RemoteMillis::new(value)?)
}
