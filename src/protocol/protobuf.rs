use prost::Message;
use serde::Serialize;

use crate::protocol::{
    Action, ClientInfoMessage, FileChunkFrame, FileDeleteRequest, FileGetRequest,
    FileRenameRequest, FileSyncRequest, FileUploadCheckRequest, FolderCreateRequest,
    FolderDeleteRequest, FolderRenameRequest, FolderSyncRequest, NoteDeleteRequest, NoteGetRequest,
    NoteModifyOrCreateRequest, NoteRenameRequest, NoteSyncRequest, NoteUpdateCheckRequest,
    ProtocolError, Result, SettingClearRequest, SettingDeleteRequest, SettingGetRequest,
    SettingModifyOrCreateRequest, SettingSyncRequest, SettingUpdateCheckRequest, TextFrame,
    WsResponse,
};

pub const PROTOBUF_BINARY_PREFIX: &str = "pb";

#[derive(Clone, PartialEq, Message)]
struct WsMessage {
    #[prost(string, tag = "1")]
    message_type: String,
    #[prost(bytes = "vec", tag = "2")]
    data: Vec<u8>,
}

#[derive(Clone, PartialEq, Message)]
struct WsResponseMessage {
    #[prost(int32, tag = "1")]
    code: i32,
    #[prost(bool, tag = "2")]
    status: bool,
    #[prost(string, tag = "3")]
    message: String,
    #[prost(bytes = "vec", tag = "4")]
    data: Vec<u8>,
    #[prost(string, tag = "5")]
    details: String,
    #[prost(string, tag = "6")]
    vault: String,
    #[prost(string, tag = "7")]
    context: String,
}

#[derive(Clone, PartialEq, Message)]
struct PbClientInfoMessage {
    #[prost(string, tag = "1")]
    name: String,
    #[prost(string, tag = "2")]
    version: String,
    #[prost(string, tag = "3")]
    client_type: String,
    #[prost(bool, tag = "4")]
    is_desktop: bool,
    #[prost(bool, tag = "5")]
    is_mobile: bool,
    #[prost(bool, tag = "6")]
    is_phone: bool,
    #[prost(bool, tag = "7")]
    is_tablet: bool,
    #[prost(bool, tag = "8")]
    is_mac_os: bool,
    #[prost(bool, tag = "9")]
    is_win: bool,
    #[prost(bool, tag = "10")]
    is_linux: bool,
    #[prost(string, tag = "11")]
    offline_sync_strategy: String,
    #[prost(bool, tag = "12")]
    protobuf: bool,
}

#[derive(Clone, PartialEq, Message, Serialize)]
#[serde(rename_all = "camelCase")]
struct HistoricalVersion {
    #[prost(string, tag = "1")]
    version: String,
    #[prost(string, tag = "2")]
    changelog_content: String,
}

#[derive(Clone, PartialEq, Message, Serialize)]
#[serde(rename_all = "camelCase")]
struct CheckVersionInfo {
    #[prost(bool, tag = "1")]
    github_available: bool,
    #[prost(bool, tag = "2")]
    version_is_new: bool,
    #[prost(string, tag = "3")]
    version_new_name: String,
    #[prost(string, tag = "4")]
    version_new_link: String,
    #[prost(string, tag = "5")]
    version_new_changelog: String,
    #[prost(string, tag = "6")]
    version_new_changelog_content: String,
    #[prost(message, repeated, tag = "7")]
    version_history: Vec<HistoricalVersion>,
    #[prost(bool, tag = "8")]
    plugin_version_is_new: bool,
    #[prost(string, tag = "9")]
    plugin_version_new_name: String,
    #[prost(string, tag = "10")]
    plugin_version_new_link: String,
    #[prost(string, tag = "11")]
    plugin_version_new_changelog: String,
    #[prost(string, tag = "12")]
    plugin_version_new_changelog_content: String,
    #[prost(message, repeated, tag = "13")]
    plugin_version_history: Vec<HistoricalVersion>,
}

#[derive(Clone, PartialEq, Message)]
struct NoteSyncCheckRequest {
    #[prost(string, tag = "1")]
    path: String,
    #[prost(string, tag = "2")]
    path_hash: String,
    #[prost(string, tag = "3")]
    content_hash: String,
    #[prost(int64, tag = "4")]
    mtime: i64,
    #[prost(int64, tag = "5")]
    ctime: i64,
}

#[derive(Clone, PartialEq, Message)]
struct NoteSyncDelNote {
    #[prost(string, tag = "1")]
    path: String,
    #[prost(string, tag = "2")]
    path_hash: String,
}

#[derive(Clone, PartialEq, Message)]
struct PbNoteSyncRequest {
    #[prost(string, tag = "1")]
    context: String,
    #[prost(string, tag = "2")]
    vault: String,
    #[prost(int64, tag = "3")]
    last_time: i64,
    #[prost(message, repeated, tag = "4")]
    notes: Vec<NoteSyncCheckRequest>,
    #[prost(message, repeated, tag = "5")]
    del_notes: Vec<NoteSyncDelNote>,
    #[prost(message, repeated, tag = "6")]
    missing_notes: Vec<NoteSyncDelNote>,
}

#[derive(Clone, PartialEq, Message)]
struct PbNoteModifyOrCreateRequest {
    #[prost(string, tag = "1")]
    vault: String,
    #[prost(string, tag = "2")]
    path: String,
    #[prost(string, tag = "3")]
    path_hash: String,
    #[prost(string, tag = "4")]
    base_hash: String,
    #[prost(bool, tag = "5")]
    base_hash_missing: bool,
    #[prost(string, tag = "6")]
    content: String,
    #[prost(string, tag = "7")]
    content_hash: String,
    #[prost(int64, tag = "8")]
    ctime: i64,
    #[prost(int64, tag = "9")]
    mtime: i64,
    #[prost(bool, tag = "10")]
    create_only: bool,
}

#[derive(Clone, PartialEq, Message)]
struct PbNoteUpdateCheckRequest {
    #[prost(string, tag = "1")]
    vault: String,
    #[prost(string, tag = "2")]
    path: String,
    #[prost(string, tag = "3")]
    path_hash: String,
    #[prost(string, tag = "4")]
    content_hash: String,
    #[prost(int64, tag = "5")]
    ctime: i64,
    #[prost(int64, tag = "6")]
    mtime: i64,
}

#[derive(Clone, PartialEq, Message)]
struct PbNoteDeleteRequest {
    #[prost(string, tag = "1")]
    vault: String,
    #[prost(string, tag = "2")]
    path: String,
    #[prost(string, tag = "3")]
    path_hash: String,
}

#[derive(Clone, PartialEq, Message)]
struct PbNoteRenameRequest {
    #[prost(string, tag = "1")]
    vault: String,
    #[prost(string, tag = "2")]
    path: String,
    #[prost(string, tag = "3")]
    path_hash: String,
    #[prost(string, tag = "4")]
    old_path: String,
    #[prost(string, tag = "5")]
    old_path_hash: String,
}

#[derive(Clone, PartialEq, Message)]
struct PbNoteGetRequest {
    #[prost(string, tag = "1")]
    vault: String,
    #[prost(string, tag = "2")]
    path: String,
    #[prost(string, tag = "3")]
    path_hash: String,
    #[prost(bool, tag = "4")]
    is_recycle: bool,
}

#[derive(Clone, PartialEq, Message, Serialize)]
#[serde(rename_all = "camelCase")]
struct PbNoteSyncModifyMessage {
    #[prost(string, tag = "1")]
    path: String,
    #[prost(string, tag = "2")]
    path_hash: String,
    #[prost(string, tag = "3")]
    content: String,
    #[prost(string, tag = "4")]
    content_hash: String,
    #[prost(int64, tag = "5")]
    ctime: i64,
    #[prost(int64, tag = "6")]
    mtime: i64,
    #[prost(int64, tag = "7")]
    last_time: i64,
}

#[derive(Clone, PartialEq, Message, Serialize)]
#[serde(rename_all = "camelCase")]
struct PbNoteSyncDeleteMessage {
    #[prost(string, tag = "1")]
    path: String,
    #[prost(string, tag = "2")]
    path_hash: String,
    #[prost(int64, tag = "3")]
    ctime: i64,
    #[prost(int64, tag = "4")]
    mtime: i64,
    #[prost(int64, tag = "5")]
    size: i64,
    #[prost(int64, tag = "6")]
    last_time: i64,
}

#[derive(Clone, PartialEq, Message, Serialize)]
#[serde(rename_all = "camelCase")]
struct PbNoteSyncRenameMessage {
    #[prost(string, tag = "1")]
    path: String,
    #[prost(string, tag = "2")]
    path_hash: String,
    #[prost(string, tag = "3")]
    content_hash: String,
    #[prost(int64, tag = "4")]
    ctime: i64,
    #[prost(int64, tag = "5")]
    mtime: i64,
    #[prost(int64, tag = "6")]
    size: i64,
    #[prost(string, tag = "7")]
    old_path: String,
    #[prost(string, tag = "8")]
    old_path_hash: String,
    #[prost(int64, tag = "9")]
    last_time: i64,
}

#[derive(Clone, PartialEq, Message, Serialize)]
#[serde(rename_all = "camelCase")]
struct PbNoteSyncMtimeMessage {
    #[prost(string, tag = "1")]
    path: String,
    #[prost(int64, tag = "2")]
    ctime: i64,
    #[prost(int64, tag = "3")]
    mtime: i64,
    #[prost(int64, tag = "4")]
    last_time: i64,
}

#[derive(Clone, PartialEq, Message, Serialize)]
#[serde(rename_all = "camelCase")]
struct PbNoteSyncEndMessage {
    #[prost(int64, tag = "1")]
    last_time: i64,
    #[prost(int64, tag = "2")]
    need_upload_count: i64,
    #[prost(int64, tag = "3")]
    need_modify_count: i64,
    #[prost(int64, tag = "4")]
    need_sync_mtime_count: i64,
    #[prost(int64, tag = "5")]
    need_delete_count: i64,
}

#[derive(Clone, PartialEq, Message, Serialize)]
#[serde(rename_all = "camelCase")]
struct PbNoteSyncNeedPushMessage {
    #[prost(string, tag = "1")]
    path: String,
    #[prost(string, tag = "2")]
    path_hash: String,
}

#[derive(Clone, PartialEq, Message, Serialize)]
#[serde(rename_all = "camelCase")]
struct PbAckMessage {
    #[prost(int64, tag = "1")]
    last_time: i64,
    #[prost(string, tag = "2")]
    path: String,
    #[prost(string, tag = "3")]
    path_hash: String,
}

#[derive(Clone, PartialEq, Message)]
struct FileSyncCheckRequest {
    #[prost(string, tag = "1")]
    path: String,
    #[prost(string, tag = "2")]
    path_hash: String,
    #[prost(string, tag = "3")]
    content_hash: String,
    #[prost(int64, tag = "4")]
    size: i64,
    #[prost(int64, tag = "5")]
    mtime: i64,
    #[prost(int64, tag = "6")]
    ctime: i64,
}

#[derive(Clone, PartialEq, Message)]
struct FileSyncDelFile {
    #[prost(string, tag = "1")]
    path: String,
    #[prost(string, tag = "2")]
    path_hash: String,
}

#[derive(Clone, PartialEq, Message)]
struct PbFileSyncRequest {
    #[prost(string, tag = "1")]
    context: String,
    #[prost(string, tag = "2")]
    vault: String,
    #[prost(int64, tag = "3")]
    last_time: i64,
    #[prost(message, repeated, tag = "4")]
    files: Vec<FileSyncCheckRequest>,
    #[prost(message, repeated, tag = "5")]
    del_files: Vec<FileSyncDelFile>,
    #[prost(message, repeated, tag = "6")]
    missing_files: Vec<FileSyncDelFile>,
}

#[derive(Clone, PartialEq, Message)]
struct PbFileUploadCheckRequest {
    #[prost(string, tag = "1")]
    vault: String,
    #[prost(string, tag = "2")]
    path: String,
    #[prost(string, tag = "3")]
    path_hash: String,
    #[prost(string, tag = "4")]
    content_hash: String,
    #[prost(int64, tag = "5")]
    size: i64,
    #[prost(int64, tag = "6")]
    ctime: i64,
    #[prost(int64, tag = "7")]
    mtime: i64,
}

#[derive(Clone, PartialEq, Message)]
struct PbFileDeleteRequest {
    #[prost(string, tag = "1")]
    vault: String,
    #[prost(string, tag = "2")]
    path: String,
    #[prost(string, tag = "3")]
    path_hash: String,
}

#[derive(Clone, PartialEq, Message)]
struct PbFileRenameRequest {
    #[prost(string, tag = "1")]
    vault: String,
    #[prost(string, tag = "2")]
    path: String,
    #[prost(string, tag = "3")]
    path_hash: String,
    #[prost(string, tag = "4")]
    old_path: String,
    #[prost(string, tag = "5")]
    old_path_hash: String,
}

#[derive(Clone, PartialEq, Message)]
struct PbFileChunkDownloadRequest {
    #[prost(string, tag = "1")]
    vault: String,
    #[prost(string, tag = "2")]
    path: String,
    #[prost(string, tag = "3")]
    path_hash: String,
    #[prost(string, tag = "4")]
    session_id: String,
    #[prost(int64, tag = "5")]
    chunk_index: i64,
}

#[derive(Clone, PartialEq, Message)]
struct PbFileGetRequest {
    #[prost(string, tag = "1")]
    vault: String,
    #[prost(string, tag = "2")]
    path: String,
    #[prost(string, tag = "3")]
    path_hash: String,
}

#[derive(Clone, PartialEq, Message, Serialize)]
#[serde(rename_all = "camelCase")]
struct PbFileSyncModifyMessage {
    #[prost(string, tag = "1")]
    path: String,
    #[prost(string, tag = "2")]
    path_hash: String,
    #[prost(string, tag = "3")]
    content_hash: String,
    #[prost(int64, tag = "4")]
    size: i64,
    #[prost(int64, tag = "5")]
    ctime: i64,
    #[prost(int64, tag = "6")]
    mtime: i64,
    #[prost(int64, tag = "7")]
    last_time: i64,
}

#[derive(Clone, PartialEq, Message, Serialize)]
#[serde(rename_all = "camelCase")]
struct PbFileSyncDeleteMessage {
    #[prost(string, tag = "1")]
    path: String,
    #[prost(string, tag = "2")]
    path_hash: String,
    #[prost(int64, tag = "3")]
    ctime: i64,
    #[prost(int64, tag = "4")]
    mtime: i64,
    #[prost(int64, tag = "5")]
    size: i64,
    #[prost(int64, tag = "6")]
    last_time: i64,
}

#[derive(Clone, PartialEq, Message, Serialize)]
#[serde(rename_all = "camelCase")]
struct PbFileSyncRenameMessage {
    #[prost(string, tag = "1")]
    path: String,
    #[prost(string, tag = "2")]
    path_hash: String,
    #[prost(string, tag = "3")]
    content_hash: String,
    #[prost(int64, tag = "4")]
    ctime: i64,
    #[prost(int64, tag = "5")]
    mtime: i64,
    #[prost(int64, tag = "6")]
    size: i64,
    #[prost(int64, tag = "7")]
    last_time: i64,
    #[prost(string, tag = "8")]
    old_path: String,
    #[prost(string, tag = "9")]
    old_path_hash: String,
}

#[derive(Clone, PartialEq, Message, Serialize)]
#[serde(rename_all = "camelCase")]
struct PbFileSyncMtimeMessage {
    #[prost(string, tag = "1")]
    path: String,
    #[prost(int64, tag = "2")]
    ctime: i64,
    #[prost(int64, tag = "3")]
    mtime: i64,
    #[prost(int64, tag = "4")]
    last_time: i64,
}

#[derive(Clone, PartialEq, Message, Serialize)]
#[serde(rename_all = "camelCase")]
struct PbFileSyncUploadMessage {
    #[prost(string, tag = "1")]
    path: String,
    #[prost(string, tag = "2")]
    path_hash: String,
    #[prost(string, tag = "3")]
    session_id: String,
    #[prost(int64, tag = "4")]
    chunk_size: i64,
}

#[derive(Clone, PartialEq, Message, Serialize)]
#[serde(rename_all = "camelCase")]
struct PbFileSyncDownloadMessage {
    #[prost(string, tag = "1")]
    path: String,
    #[prost(string, tag = "2")]
    content_hash: String,
    #[prost(int64, tag = "3")]
    ctime: i64,
    #[prost(int64, tag = "4")]
    mtime: i64,
    #[prost(string, tag = "5")]
    session_id: String,
    #[prost(int64, tag = "6")]
    chunk_size: i64,
    #[prost(int64, tag = "7")]
    total_chunks: i64,
    #[prost(int64, tag = "8")]
    size: i64,
}

#[derive(Clone, PartialEq, Message)]
struct SettingSyncCheckRequest {
    #[prost(string, tag = "1")]
    path: String,
    #[prost(string, tag = "2")]
    path_hash: String,
    #[prost(string, tag = "3")]
    content_hash: String,
    #[prost(int64, tag = "4")]
    mtime: i64,
    #[prost(int64, tag = "5")]
    ctime: i64,
}

#[derive(Clone, PartialEq, Message)]
struct SettingSyncDelSetting {
    #[prost(string, tag = "1")]
    path: String,
    #[prost(string, tag = "2")]
    path_hash: String,
}

#[derive(Clone, PartialEq, Message)]
struct PbSettingSyncRequest {
    #[prost(string, tag = "1")]
    context: String,
    #[prost(string, tag = "2")]
    vault: String,
    #[prost(int64, tag = "3")]
    last_time: i64,
    #[prost(message, repeated, tag = "4")]
    settings: Vec<SettingSyncCheckRequest>,
    #[prost(message, repeated, tag = "5")]
    del_settings: Vec<SettingSyncDelSetting>,
    #[prost(message, repeated, tag = "6")]
    missing_settings: Vec<SettingSyncDelSetting>,
}

#[derive(Clone, PartialEq, Message)]
struct PbSettingModifyOrCreateRequest {
    #[prost(string, tag = "1")]
    vault: String,
    #[prost(string, tag = "2")]
    path: String,
    #[prost(string, tag = "3")]
    path_hash: String,
    #[prost(string, tag = "4")]
    content: String,
    #[prost(string, tag = "5")]
    content_hash: String,
    #[prost(int64, tag = "6")]
    ctime: i64,
    #[prost(int64, tag = "7")]
    mtime: i64,
}

#[derive(Clone, PartialEq, Message)]
struct PbSettingUpdateCheckRequest {
    #[prost(string, tag = "1")]
    vault: String,
    #[prost(string, tag = "2")]
    path: String,
    #[prost(string, tag = "3")]
    path_hash: String,
    #[prost(string, tag = "4")]
    content_hash: String,
    #[prost(int64, tag = "5")]
    ctime: i64,
    #[prost(int64, tag = "6")]
    mtime: i64,
}

#[derive(Clone, PartialEq, Message)]
struct PbSettingDeleteRequest {
    #[prost(string, tag = "1")]
    vault: String,
    #[prost(string, tag = "2")]
    path: String,
    #[prost(string, tag = "3")]
    path_hash: String,
}

#[derive(Clone, PartialEq, Message)]
struct PbSettingGetRequest {
    #[prost(string, tag = "1")]
    vault: String,
    #[prost(string, tag = "2")]
    path: String,
    #[prost(string, tag = "3")]
    path_hash: String,
}

#[derive(Clone, PartialEq, Message)]
struct PbSettingClearRequest {
    #[prost(string, tag = "1")]
    vault: String,
}

#[derive(Clone, PartialEq, Message, Serialize)]
#[serde(rename_all = "camelCase")]
struct PbSettingSyncModifyMessage {
    #[prost(string, tag = "1")]
    vault: String,
    #[prost(string, tag = "2")]
    path: String,
    #[prost(string, tag = "3")]
    path_hash: String,
    #[prost(string, tag = "4")]
    content: String,
    #[prost(string, tag = "5")]
    content_hash: String,
    #[prost(int64, tag = "6")]
    ctime: i64,
    #[prost(int64, tag = "7")]
    mtime: i64,
    #[prost(int64, tag = "8")]
    last_time: i64,
}

#[derive(Clone, PartialEq, Message, Serialize)]
#[serde(rename_all = "camelCase")]
struct PbSettingSyncDeleteMessage {
    #[prost(string, tag = "1")]
    path: String,
    #[prost(string, tag = "2")]
    path_hash: String,
    #[prost(int64, tag = "3")]
    ctime: i64,
    #[prost(int64, tag = "4")]
    mtime: i64,
    #[prost(int64, tag = "5")]
    last_time: i64,
}

#[derive(Clone, PartialEq, Message, Serialize)]
#[serde(rename_all = "camelCase")]
struct PbSettingSyncMtimeMessage {
    #[prost(string, tag = "1")]
    path: String,
    #[prost(int64, tag = "2")]
    ctime: i64,
    #[prost(int64, tag = "3")]
    mtime: i64,
    #[prost(int64, tag = "4")]
    last_time: i64,
}

#[derive(Clone, PartialEq, Message, Serialize)]
#[serde(rename_all = "camelCase")]
struct PbSettingSyncNeedUploadMessage {
    #[prost(string, tag = "1")]
    path: String,
}

#[derive(Clone, PartialEq, Message)]
struct FolderSyncCheckRequest {
    #[prost(string, tag = "1")]
    path: String,
    #[prost(string, tag = "2")]
    path_hash: String,
    #[prost(int64, tag = "3")]
    mtime: i64,
}

#[derive(Clone, PartialEq, Message)]
struct FolderSyncDelFolder {
    #[prost(string, tag = "1")]
    path: String,
    #[prost(string, tag = "2")]
    path_hash: String,
}

#[derive(Clone, PartialEq, Message)]
struct PbFolderSyncRequest {
    #[prost(string, tag = "1")]
    context: String,
    #[prost(string, tag = "2")]
    vault: String,
    #[prost(int64, tag = "3")]
    last_time: i64,
    #[prost(message, repeated, tag = "4")]
    folders: Vec<FolderSyncCheckRequest>,
    #[prost(message, repeated, tag = "5")]
    del_folders: Vec<FolderSyncDelFolder>,
    #[prost(message, repeated, tag = "6")]
    missing_folders: Vec<FolderSyncDelFolder>,
}

#[derive(Clone, PartialEq, Message)]
struct PbFolderCreateRequest {
    #[prost(string, tag = "1")]
    vault: String,
    #[prost(string, tag = "2")]
    path: String,
    #[prost(string, tag = "3")]
    path_hash: String,
}

#[derive(Clone, PartialEq, Message)]
struct PbFolderDeleteRequest {
    #[prost(string, tag = "1")]
    vault: String,
    #[prost(string, tag = "2")]
    path: String,
    #[prost(string, tag = "3")]
    path_hash: String,
}

#[derive(Clone, PartialEq, Message)]
struct PbFolderRenameRequest {
    #[prost(string, tag = "1")]
    vault: String,
    #[prost(string, tag = "2")]
    path: String,
    #[prost(string, tag = "3")]
    path_hash: String,
    #[prost(string, tag = "4")]
    old_path: String,
    #[prost(string, tag = "5")]
    old_path_hash: String,
}

#[derive(Clone, PartialEq, Message, Serialize)]
#[serde(rename_all = "camelCase")]
struct PbFolderSyncModifyMessage {
    #[prost(string, tag = "1")]
    path: String,
    #[prost(string, tag = "2")]
    path_hash: String,
    #[prost(int64, tag = "3")]
    ctime: i64,
    #[prost(int64, tag = "4")]
    mtime: i64,
    #[prost(int64, tag = "5")]
    last_time: i64,
}

#[derive(Clone, PartialEq, Message, Serialize)]
#[serde(rename_all = "camelCase")]
struct PbFolderSyncDeleteMessage {
    #[prost(string, tag = "1")]
    path: String,
    #[prost(string, tag = "2")]
    path_hash: String,
    #[prost(int64, tag = "3")]
    ctime: i64,
    #[prost(int64, tag = "4")]
    mtime: i64,
    #[prost(int64, tag = "5")]
    last_time: i64,
}

#[derive(Clone, PartialEq, Message, Serialize)]
#[serde(rename_all = "camelCase")]
struct PbFolderSyncRenameMessage {
    #[prost(string, tag = "1")]
    path: String,
    #[prost(string, tag = "2")]
    path_hash: String,
    #[prost(int64, tag = "3")]
    ctime: i64,
    #[prost(int64, tag = "4")]
    mtime: i64,
    #[prost(string, tag = "5")]
    old_path: String,
    #[prost(string, tag = "6")]
    old_path_hash: String,
    #[prost(int64, tag = "7")]
    last_time: i64,
}

#[derive(Clone, PartialEq, Message, Serialize)]
#[serde(rename_all = "camelCase")]
struct PbFolderSyncEndMessage {
    #[prost(int64, tag = "1")]
    last_time: i64,
    #[prost(int64, tag = "2")]
    need_modify_count: i64,
    #[prost(int64, tag = "3")]
    need_delete_count: i64,
}

pub fn encode_protobuf_frame<T>(action: Action, payload: &T) -> Result<Vec<u8>>
where
    T: Serialize,
{
    let payload = serde_json::to_value(payload)?;
    let data = encode_request_payload(&action, payload)?;
    encode_ws_message(action, data)
}

pub fn decode_protobuf_frame(frame: &[u8]) -> Result<TextFrame> {
    let envelope = WsMessage::decode(frame)?;
    let action = Action::try_from(envelope.message_type)?;
    let response = WsResponseMessage::decode(envelope.data.as_slice())?;
    let data = decode_response_payload(&action, &response.data)?;

    let response = WsResponse {
        code: response.code,
        status: response.status,
        message: response.message,
        data,
        details: optional_string(response.details),
        vault: optional_string(response.vault),
        context: optional_string(response.context),
    };
    Ok(TextFrame::new(action, serde_json::to_string(&response)?))
}

fn encode_ws_message(action: Action, data: Vec<u8>) -> Result<Vec<u8>> {
    let envelope = WsMessage {
        message_type: action.as_str().to_string(),
        data,
    };
    let envelope = envelope.encode_to_vec();
    let mut frame = Vec::with_capacity(2 + envelope.len());
    frame.extend_from_slice(PROTOBUF_BINARY_PREFIX.as_bytes());
    frame.extend_from_slice(&envelope);
    Ok(frame)
}

fn encode_request_payload(action: &Action, payload: serde_json::Value) -> Result<Vec<u8>> {
    macro_rules! encode {
        ($ty:ty, $pb:ty) => {{
            let value: $ty = serde_json::from_value(payload)?;
            Ok(<$pb>::from(value).encode_to_vec())
        }};
    }

    match action {
        Action::ClientInfo => encode!(ClientInfoMessage, PbClientInfoMessage),
        Action::FolderSync => encode!(FolderSyncRequest, PbFolderSyncRequest),
        Action::FolderModify => encode!(FolderCreateRequest, PbFolderCreateRequest),
        Action::FolderDelete => encode!(FolderDeleteRequest, PbFolderDeleteRequest),
        Action::FolderRename => encode!(FolderRenameRequest, PbFolderRenameRequest),
        Action::NoteSync => encode!(NoteSyncRequest, PbNoteSyncRequest),
        Action::NoteModify => encode!(NoteModifyOrCreateRequest, PbNoteModifyOrCreateRequest),
        Action::NoteCheck => encode!(NoteUpdateCheckRequest, PbNoteUpdateCheckRequest),
        Action::NoteDelete => encode!(NoteDeleteRequest, PbNoteDeleteRequest),
        Action::NoteRename => encode!(NoteRenameRequest, PbNoteRenameRequest),
        Action::NoteRePush => encode!(NoteGetRequest, PbNoteGetRequest),
        Action::FileSync => encode!(FileSyncRequest, PbFileSyncRequest),
        Action::FileUploadCheck => encode!(FileUploadCheckRequest, PbFileUploadCheckRequest),
        Action::FileDelete => encode!(FileDeleteRequest, PbFileDeleteRequest),
        Action::FileRename => encode!(FileRenameRequest, PbFileRenameRequest),
        Action::FileChunkDownload => encode!(FileGetRequest, PbFileChunkDownloadRequest),
        Action::FileRePush => encode!(FileGetRequest, PbFileGetRequest),
        Action::SettingSync => encode!(SettingSyncRequest, PbSettingSyncRequest),
        Action::SettingModify => {
            encode!(SettingModifyOrCreateRequest, PbSettingModifyOrCreateRequest)
        }
        Action::SettingCheck => encode!(SettingUpdateCheckRequest, PbSettingUpdateCheckRequest),
        Action::SettingDelete => encode!(SettingDeleteRequest, PbSettingDeleteRequest),
        Action::SettingClear => encode!(SettingClearRequest, PbSettingClearRequest),
        Action::SettingRePush => encode!(SettingGetRequest, PbSettingGetRequest),
        _ => Err(ProtocolError::UnsupportedProtobufAction(action.to_string())),
    }
}

fn decode_response_payload(action: &Action, data: &[u8]) -> Result<Option<serde_json::Value>> {
    if data.is_empty() {
        return Ok(None);
    }

    macro_rules! decode {
        ($ty:ty) => {{
            let decoded = <$ty>::decode(data)?;
            Some(serde_json::to_value(decoded)?)
        }};
    }

    let data = match action {
        Action::ClientInfo => decode!(CheckVersionInfo),
        Action::NoteSyncModify => decode!(PbNoteSyncModifyMessage),
        Action::NoteSyncDelete => decode!(PbNoteSyncDeleteMessage),
        Action::NoteSyncRename => decode!(PbNoteSyncRenameMessage),
        Action::NoteSyncMtime => decode!(PbNoteSyncMtimeMessage),
        Action::NoteSyncEnd => decode!(PbNoteSyncEndMessage),
        Action::NoteSyncNeedPush => decode!(PbNoteSyncNeedPushMessage),
        Action::NoteModifyAck | Action::NoteRenameAck | Action::NoteDeleteAck => {
            decode!(PbAckMessage)
        }
        Action::FileSyncUpdate => decode!(PbFileSyncModifyMessage),
        Action::FileSyncDelete => decode!(PbFileSyncDeleteMessage),
        Action::FileSyncRename => decode!(PbFileSyncRenameMessage),
        Action::FileSyncMtime => decode!(PbFileSyncMtimeMessage),
        Action::FileSyncEnd => decode!(PbNoteSyncEndMessage),
        Action::FileUpload => decode!(PbFileSyncUploadMessage),
        Action::FileSyncChunkDownload => decode!(PbFileSyncDownloadMessage),
        Action::FileUploadAck | Action::FileRenameAck | Action::FileDeleteAck => {
            decode!(PbAckMessage)
        }
        Action::SettingSyncModify => decode!(PbSettingSyncModifyMessage),
        Action::SettingSyncDelete => decode!(PbSettingSyncDeleteMessage),
        Action::SettingSyncMtime => decode!(PbSettingSyncMtimeMessage),
        Action::SettingSyncEnd => decode!(PbNoteSyncEndMessage),
        Action::SettingSyncNeedUpload => decode!(PbSettingSyncNeedUploadMessage),
        Action::SettingModifyAck | Action::SettingDeleteAck => decode!(PbAckMessage),
        Action::SettingSyncClear => None,
        Action::FolderSyncModify => decode!(PbFolderSyncModifyMessage),
        Action::FolderSyncDelete => decode!(PbFolderSyncDeleteMessage),
        Action::FolderSyncRename => decode!(PbFolderSyncRenameMessage),
        Action::FolderSyncEnd => decode!(PbFolderSyncEndMessage),
        Action::FolderModifyAck | Action::FolderRenameAck | Action::FolderDeleteAck => {
            decode!(PbAckMessage)
        }
        _ => return Err(ProtocolError::UnsupportedProtobufAction(action.to_string())),
    };

    Ok(data)
}

fn optional_string(value: String) -> Option<String> {
    if value.is_empty() { None } else { Some(value) }
}

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
