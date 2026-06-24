use prost::Message;
use serde::Serialize;

pub const PROTOBUF_BINARY_PREFIX: &str = "pb";

#[derive(Clone, PartialEq, Message)]
pub(super) struct WsMessage {
    #[prost(string, tag = "1")]
    pub(super) message_type: String,
    #[prost(bytes = "vec", tag = "2")]
    pub(super) data: Vec<u8>,
}

#[derive(Clone, PartialEq, Message)]
pub(super) struct WsResponseMessage {
    #[prost(int32, tag = "1")]
    pub(super) code: i32,
    #[prost(bool, tag = "2")]
    pub(super) status: bool,
    #[prost(string, tag = "3")]
    pub(super) message: String,
    #[prost(bytes = "vec", tag = "4")]
    pub(super) data: Vec<u8>,
    #[prost(string, tag = "5")]
    pub(super) details: String,
    #[prost(string, tag = "6")]
    pub(super) vault: String,
    #[prost(string, tag = "7")]
    pub(super) context: String,
}

#[derive(Clone, PartialEq, Message)]
pub(super) struct PbClientInfoMessage {
    #[prost(string, tag = "1")]
    pub(super) name: String,
    #[prost(string, tag = "2")]
    pub(super) version: String,
    #[prost(string, tag = "3")]
    pub(super) client_type: String,
    #[prost(bool, tag = "4")]
    pub(super) is_desktop: bool,
    #[prost(bool, tag = "5")]
    pub(super) is_mobile: bool,
    #[prost(bool, tag = "6")]
    pub(super) is_phone: bool,
    #[prost(bool, tag = "7")]
    pub(super) is_tablet: bool,
    #[prost(bool, tag = "8")]
    pub(super) is_mac_os: bool,
    #[prost(bool, tag = "9")]
    pub(super) is_win: bool,
    #[prost(bool, tag = "10")]
    pub(super) is_linux: bool,
    #[prost(string, tag = "11")]
    pub(super) offline_sync_strategy: String,
    #[prost(bool, tag = "12")]
    pub(super) protobuf: bool,
}

#[derive(Clone, PartialEq, Message, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct HistoricalVersion {
    #[prost(string, tag = "1")]
    pub(super) version: String,
    #[prost(string, tag = "2")]
    pub(super) changelog_content: String,
}

#[derive(Clone, PartialEq, Message, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CheckVersionInfo {
    #[prost(bool, tag = "1")]
    pub(super) github_available: bool,
    #[prost(bool, tag = "2")]
    pub(super) version_is_new: bool,
    #[prost(string, tag = "3")]
    pub(super) version_new_name: String,
    #[prost(string, tag = "4")]
    pub(super) version_new_link: String,
    #[prost(string, tag = "5")]
    pub(super) version_new_changelog: String,
    #[prost(string, tag = "6")]
    pub(super) version_new_changelog_content: String,
    #[prost(message, repeated, tag = "7")]
    pub(super) version_history: Vec<HistoricalVersion>,
    #[prost(bool, tag = "8")]
    pub(super) plugin_version_is_new: bool,
    #[prost(string, tag = "9")]
    pub(super) plugin_version_new_name: String,
    #[prost(string, tag = "10")]
    pub(super) plugin_version_new_link: String,
    #[prost(string, tag = "11")]
    pub(super) plugin_version_new_changelog: String,
    #[prost(string, tag = "12")]
    pub(super) plugin_version_new_changelog_content: String,
    #[prost(message, repeated, tag = "13")]
    pub(super) plugin_version_history: Vec<HistoricalVersion>,
}

#[derive(Clone, PartialEq, Message)]
pub(super) struct NoteSyncCheckRequest {
    #[prost(string, tag = "1")]
    pub(super) path: String,
    #[prost(string, tag = "2")]
    pub(super) path_hash: String,
    #[prost(string, tag = "3")]
    pub(super) content_hash: String,
    #[prost(int64, tag = "4")]
    pub(super) mtime: i64,
    #[prost(int64, tag = "5")]
    pub(super) ctime: i64,
}

#[derive(Clone, PartialEq, Message)]
pub(super) struct NoteSyncDelNote {
    #[prost(string, tag = "1")]
    pub(super) path: String,
    #[prost(string, tag = "2")]
    pub(super) path_hash: String,
}

#[derive(Clone, PartialEq, Message)]
pub(super) struct PbNoteSyncRequest {
    #[prost(string, tag = "1")]
    pub(super) context: String,
    #[prost(string, tag = "2")]
    pub(super) vault: String,
    #[prost(int64, tag = "3")]
    pub(super) last_time: i64,
    #[prost(message, repeated, tag = "4")]
    pub(super) notes: Vec<NoteSyncCheckRequest>,
    #[prost(message, repeated, tag = "5")]
    pub(super) del_notes: Vec<NoteSyncDelNote>,
    #[prost(message, repeated, tag = "6")]
    pub(super) missing_notes: Vec<NoteSyncDelNote>,
}

#[derive(Clone, PartialEq, Message)]
pub(super) struct PbNoteModifyOrCreateRequest {
    #[prost(string, tag = "1")]
    pub(super) vault: String,
    #[prost(string, tag = "2")]
    pub(super) path: String,
    #[prost(string, tag = "3")]
    pub(super) path_hash: String,
    #[prost(string, tag = "4")]
    pub(super) base_hash: String,
    #[prost(bool, tag = "5")]
    pub(super) base_hash_missing: bool,
    #[prost(string, tag = "6")]
    pub(super) content: String,
    #[prost(string, tag = "7")]
    pub(super) content_hash: String,
    #[prost(int64, tag = "8")]
    pub(super) ctime: i64,
    #[prost(int64, tag = "9")]
    pub(super) mtime: i64,
    #[prost(bool, tag = "10")]
    pub(super) create_only: bool,
}

#[derive(Clone, PartialEq, Message)]
pub(super) struct PbNoteUpdateCheckRequest {
    #[prost(string, tag = "1")]
    pub(super) vault: String,
    #[prost(string, tag = "2")]
    pub(super) path: String,
    #[prost(string, tag = "3")]
    pub(super) path_hash: String,
    #[prost(string, tag = "4")]
    pub(super) content_hash: String,
    #[prost(int64, tag = "5")]
    pub(super) ctime: i64,
    #[prost(int64, tag = "6")]
    pub(super) mtime: i64,
}

#[derive(Clone, PartialEq, Message)]
pub(super) struct PbNoteDeleteRequest {
    #[prost(string, tag = "1")]
    pub(super) vault: String,
    #[prost(string, tag = "2")]
    pub(super) path: String,
    #[prost(string, tag = "3")]
    pub(super) path_hash: String,
}

#[derive(Clone, PartialEq, Message)]
pub(super) struct PbNoteRenameRequest {
    #[prost(string, tag = "1")]
    pub(super) vault: String,
    #[prost(string, tag = "2")]
    pub(super) path: String,
    #[prost(string, tag = "3")]
    pub(super) path_hash: String,
    #[prost(string, tag = "4")]
    pub(super) old_path: String,
    #[prost(string, tag = "5")]
    pub(super) old_path_hash: String,
}

#[derive(Clone, PartialEq, Message)]
pub(super) struct PbNoteGetRequest {
    #[prost(string, tag = "1")]
    pub(super) vault: String,
    #[prost(string, tag = "2")]
    pub(super) path: String,
    #[prost(string, tag = "3")]
    pub(super) path_hash: String,
    #[prost(bool, tag = "4")]
    pub(super) is_recycle: bool,
}

#[derive(Clone, PartialEq, Message, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PbNoteSyncModifyMessage {
    #[prost(string, tag = "1")]
    pub(super) path: String,
    #[prost(string, tag = "2")]
    pub(super) path_hash: String,
    #[prost(string, tag = "3")]
    pub(super) content: String,
    #[prost(string, tag = "4")]
    pub(super) content_hash: String,
    #[prost(int64, tag = "5")]
    pub(super) ctime: i64,
    #[prost(int64, tag = "6")]
    pub(super) mtime: i64,
    #[prost(int64, tag = "7")]
    pub(super) last_time: i64,
}

#[derive(Clone, PartialEq, Message, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PbNoteSyncDeleteMessage {
    #[prost(string, tag = "1")]
    pub(super) path: String,
    #[prost(string, tag = "2")]
    pub(super) path_hash: String,
    #[prost(int64, tag = "3")]
    pub(super) ctime: i64,
    #[prost(int64, tag = "4")]
    pub(super) mtime: i64,
    #[prost(int64, tag = "5")]
    pub(super) size: i64,
    #[prost(int64, tag = "6")]
    pub(super) last_time: i64,
}

#[derive(Clone, PartialEq, Message, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PbNoteSyncRenameMessage {
    #[prost(string, tag = "1")]
    pub(super) path: String,
    #[prost(string, tag = "2")]
    pub(super) path_hash: String,
    #[prost(string, tag = "3")]
    pub(super) content_hash: String,
    #[prost(int64, tag = "4")]
    pub(super) ctime: i64,
    #[prost(int64, tag = "5")]
    pub(super) mtime: i64,
    #[prost(int64, tag = "6")]
    pub(super) size: i64,
    #[prost(string, tag = "7")]
    pub(super) old_path: String,
    #[prost(string, tag = "8")]
    pub(super) old_path_hash: String,
    #[prost(int64, tag = "9")]
    pub(super) last_time: i64,
}

#[derive(Clone, PartialEq, Message, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PbNoteSyncMtimeMessage {
    #[prost(string, tag = "1")]
    pub(super) path: String,
    #[prost(int64, tag = "2")]
    pub(super) ctime: i64,
    #[prost(int64, tag = "3")]
    pub(super) mtime: i64,
    #[prost(int64, tag = "4")]
    pub(super) last_time: i64,
}

#[derive(Clone, PartialEq, Message, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PbNoteSyncEndMessage {
    #[prost(int64, tag = "1")]
    pub(super) last_time: i64,
    #[prost(int64, tag = "2")]
    pub(super) need_upload_count: i64,
    #[prost(int64, tag = "3")]
    pub(super) need_modify_count: i64,
    #[prost(int64, tag = "4")]
    pub(super) need_sync_mtime_count: i64,
    #[prost(int64, tag = "5")]
    pub(super) need_delete_count: i64,
}

#[derive(Clone, PartialEq, Message, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PbNoteSyncNeedPushMessage {
    #[prost(string, tag = "1")]
    pub(super) path: String,
    #[prost(string, tag = "2")]
    pub(super) path_hash: String,
}

#[derive(Clone, PartialEq, Message, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PbAckMessage {
    #[prost(int64, tag = "1")]
    pub(super) last_time: i64,
    #[prost(string, tag = "2")]
    pub(super) path: String,
    #[prost(string, tag = "3")]
    pub(super) path_hash: String,
}

#[derive(Clone, PartialEq, Message)]
pub(super) struct FileSyncCheckRequest {
    #[prost(string, tag = "1")]
    pub(super) path: String,
    #[prost(string, tag = "2")]
    pub(super) path_hash: String,
    #[prost(string, tag = "3")]
    pub(super) content_hash: String,
    #[prost(int64, tag = "4")]
    pub(super) size: i64,
    #[prost(int64, tag = "5")]
    pub(super) mtime: i64,
    #[prost(int64, tag = "6")]
    pub(super) ctime: i64,
}

#[derive(Clone, PartialEq, Message)]
pub(super) struct FileSyncDelFile {
    #[prost(string, tag = "1")]
    pub(super) path: String,
    #[prost(string, tag = "2")]
    pub(super) path_hash: String,
}

#[derive(Clone, PartialEq, Message)]
pub(super) struct PbFileSyncRequest {
    #[prost(string, tag = "1")]
    pub(super) context: String,
    #[prost(string, tag = "2")]
    pub(super) vault: String,
    #[prost(int64, tag = "3")]
    pub(super) last_time: i64,
    #[prost(message, repeated, tag = "4")]
    pub(super) files: Vec<FileSyncCheckRequest>,
    #[prost(message, repeated, tag = "5")]
    pub(super) del_files: Vec<FileSyncDelFile>,
    #[prost(message, repeated, tag = "6")]
    pub(super) missing_files: Vec<FileSyncDelFile>,
}

#[derive(Clone, PartialEq, Message)]
pub(super) struct PbFileUploadCheckRequest {
    #[prost(string, tag = "1")]
    pub(super) vault: String,
    #[prost(string, tag = "2")]
    pub(super) path: String,
    #[prost(string, tag = "3")]
    pub(super) path_hash: String,
    #[prost(string, tag = "4")]
    pub(super) content_hash: String,
    #[prost(int64, tag = "5")]
    pub(super) size: i64,
    #[prost(int64, tag = "6")]
    pub(super) ctime: i64,
    #[prost(int64, tag = "7")]
    pub(super) mtime: i64,
}

#[derive(Clone, PartialEq, Message)]
pub(super) struct PbFileDeleteRequest {
    #[prost(string, tag = "1")]
    pub(super) vault: String,
    #[prost(string, tag = "2")]
    pub(super) path: String,
    #[prost(string, tag = "3")]
    pub(super) path_hash: String,
}

#[derive(Clone, PartialEq, Message)]
pub(super) struct PbFileRenameRequest {
    #[prost(string, tag = "1")]
    pub(super) vault: String,
    #[prost(string, tag = "2")]
    pub(super) path: String,
    #[prost(string, tag = "3")]
    pub(super) path_hash: String,
    #[prost(string, tag = "4")]
    pub(super) old_path: String,
    #[prost(string, tag = "5")]
    pub(super) old_path_hash: String,
}

#[derive(Clone, PartialEq, Message)]
pub(super) struct PbFileChunkDownloadRequest {
    #[prost(string, tag = "1")]
    pub(super) vault: String,
    #[prost(string, tag = "2")]
    pub(super) path: String,
    #[prost(string, tag = "3")]
    pub(super) path_hash: String,
    #[prost(string, tag = "4")]
    pub(super) session_id: String,
    #[prost(int64, tag = "5")]
    pub(super) chunk_index: i64,
}

#[derive(Clone, PartialEq, Message)]
pub(super) struct PbFileGetRequest {
    #[prost(string, tag = "1")]
    pub(super) vault: String,
    #[prost(string, tag = "2")]
    pub(super) path: String,
    #[prost(string, tag = "3")]
    pub(super) path_hash: String,
}

#[derive(Clone, PartialEq, Message, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PbFileSyncModifyMessage {
    #[prost(string, tag = "1")]
    pub(super) path: String,
    #[prost(string, tag = "2")]
    pub(super) path_hash: String,
    #[prost(string, tag = "3")]
    pub(super) content_hash: String,
    #[prost(int64, tag = "4")]
    pub(super) size: i64,
    #[prost(int64, tag = "5")]
    pub(super) ctime: i64,
    #[prost(int64, tag = "6")]
    pub(super) mtime: i64,
    #[prost(int64, tag = "7")]
    pub(super) last_time: i64,
}

#[derive(Clone, PartialEq, Message, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PbFileSyncDeleteMessage {
    #[prost(string, tag = "1")]
    pub(super) path: String,
    #[prost(string, tag = "2")]
    pub(super) path_hash: String,
    #[prost(int64, tag = "3")]
    pub(super) ctime: i64,
    #[prost(int64, tag = "4")]
    pub(super) mtime: i64,
    #[prost(int64, tag = "5")]
    pub(super) size: i64,
    #[prost(int64, tag = "6")]
    pub(super) last_time: i64,
}

#[derive(Clone, PartialEq, Message, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PbFileSyncRenameMessage {
    #[prost(string, tag = "1")]
    pub(super) path: String,
    #[prost(string, tag = "2")]
    pub(super) path_hash: String,
    #[prost(string, tag = "3")]
    pub(super) content_hash: String,
    #[prost(int64, tag = "4")]
    pub(super) ctime: i64,
    #[prost(int64, tag = "5")]
    pub(super) mtime: i64,
    #[prost(int64, tag = "6")]
    pub(super) size: i64,
    #[prost(int64, tag = "7")]
    pub(super) last_time: i64,
    #[prost(string, tag = "8")]
    pub(super) old_path: String,
    #[prost(string, tag = "9")]
    pub(super) old_path_hash: String,
}

#[derive(Clone, PartialEq, Message, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PbFileSyncMtimeMessage {
    #[prost(string, tag = "1")]
    pub(super) path: String,
    #[prost(int64, tag = "2")]
    pub(super) ctime: i64,
    #[prost(int64, tag = "3")]
    pub(super) mtime: i64,
    #[prost(int64, tag = "4")]
    pub(super) last_time: i64,
}

#[derive(Clone, PartialEq, Message, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PbFileSyncUploadMessage {
    #[prost(string, tag = "1")]
    pub(super) path: String,
    #[prost(string, tag = "2")]
    pub(super) path_hash: String,
    #[prost(string, tag = "3")]
    pub(super) session_id: String,
    #[prost(int64, tag = "4")]
    pub(super) chunk_size: i64,
}

#[derive(Clone, PartialEq, Message, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PbFileSyncDownloadMessage {
    #[prost(string, tag = "1")]
    pub(super) path: String,
    #[prost(string, tag = "2")]
    pub(super) content_hash: String,
    #[prost(int64, tag = "3")]
    pub(super) ctime: i64,
    #[prost(int64, tag = "4")]
    pub(super) mtime: i64,
    #[prost(string, tag = "5")]
    pub(super) session_id: String,
    #[prost(int64, tag = "6")]
    pub(super) chunk_size: i64,
    #[prost(int64, tag = "7")]
    pub(super) total_chunks: i64,
    #[prost(int64, tag = "8")]
    pub(super) size: i64,
}

#[derive(Clone, PartialEq, Message)]
pub(super) struct SettingSyncCheckRequest {
    #[prost(string, tag = "1")]
    pub(super) path: String,
    #[prost(string, tag = "2")]
    pub(super) path_hash: String,
    #[prost(string, tag = "3")]
    pub(super) content_hash: String,
    #[prost(int64, tag = "4")]
    pub(super) mtime: i64,
    #[prost(int64, tag = "5")]
    pub(super) ctime: i64,
}

#[derive(Clone, PartialEq, Message)]
pub(super) struct SettingSyncDelSetting {
    #[prost(string, tag = "1")]
    pub(super) path: String,
    #[prost(string, tag = "2")]
    pub(super) path_hash: String,
}

#[derive(Clone, PartialEq, Message)]
pub(super) struct PbSettingSyncRequest {
    #[prost(string, tag = "1")]
    pub(super) context: String,
    #[prost(string, tag = "2")]
    pub(super) vault: String,
    #[prost(int64, tag = "3")]
    pub(super) last_time: i64,
    #[prost(message, repeated, tag = "4")]
    pub(super) settings: Vec<SettingSyncCheckRequest>,
    #[prost(message, repeated, tag = "5")]
    pub(super) del_settings: Vec<SettingSyncDelSetting>,
    #[prost(message, repeated, tag = "6")]
    pub(super) missing_settings: Vec<SettingSyncDelSetting>,
}

#[derive(Clone, PartialEq, Message)]
pub(super) struct PbSettingModifyOrCreateRequest {
    #[prost(string, tag = "1")]
    pub(super) vault: String,
    #[prost(string, tag = "2")]
    pub(super) path: String,
    #[prost(string, tag = "3")]
    pub(super) path_hash: String,
    #[prost(string, tag = "4")]
    pub(super) content: String,
    #[prost(string, tag = "5")]
    pub(super) content_hash: String,
    #[prost(int64, tag = "6")]
    pub(super) ctime: i64,
    #[prost(int64, tag = "7")]
    pub(super) mtime: i64,
}

#[derive(Clone, PartialEq, Message)]
pub(super) struct PbSettingUpdateCheckRequest {
    #[prost(string, tag = "1")]
    pub(super) vault: String,
    #[prost(string, tag = "2")]
    pub(super) path: String,
    #[prost(string, tag = "3")]
    pub(super) path_hash: String,
    #[prost(string, tag = "4")]
    pub(super) content_hash: String,
    #[prost(int64, tag = "5")]
    pub(super) ctime: i64,
    #[prost(int64, tag = "6")]
    pub(super) mtime: i64,
}

#[derive(Clone, PartialEq, Message)]
pub(super) struct PbSettingDeleteRequest {
    #[prost(string, tag = "1")]
    pub(super) vault: String,
    #[prost(string, tag = "2")]
    pub(super) path: String,
    #[prost(string, tag = "3")]
    pub(super) path_hash: String,
}

#[derive(Clone, PartialEq, Message)]
pub(super) struct PbSettingGetRequest {
    #[prost(string, tag = "1")]
    pub(super) vault: String,
    #[prost(string, tag = "2")]
    pub(super) path: String,
    #[prost(string, tag = "3")]
    pub(super) path_hash: String,
}

#[derive(Clone, PartialEq, Message)]
pub(super) struct PbSettingClearRequest {
    #[prost(string, tag = "1")]
    pub(super) vault: String,
}

#[derive(Clone, PartialEq, Message, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PbSettingSyncModifyMessage {
    #[prost(string, tag = "1")]
    pub(super) vault: String,
    #[prost(string, tag = "2")]
    pub(super) path: String,
    #[prost(string, tag = "3")]
    pub(super) path_hash: String,
    #[prost(string, tag = "4")]
    pub(super) content: String,
    #[prost(string, tag = "5")]
    pub(super) content_hash: String,
    #[prost(int64, tag = "6")]
    pub(super) ctime: i64,
    #[prost(int64, tag = "7")]
    pub(super) mtime: i64,
    #[prost(int64, tag = "8")]
    pub(super) last_time: i64,
}

#[derive(Clone, PartialEq, Message, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PbSettingSyncDeleteMessage {
    #[prost(string, tag = "1")]
    pub(super) path: String,
    #[prost(string, tag = "2")]
    pub(super) path_hash: String,
    #[prost(int64, tag = "3")]
    pub(super) ctime: i64,
    #[prost(int64, tag = "4")]
    pub(super) mtime: i64,
    #[prost(int64, tag = "5")]
    pub(super) last_time: i64,
}

#[derive(Clone, PartialEq, Message, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PbSettingSyncMtimeMessage {
    #[prost(string, tag = "1")]
    pub(super) path: String,
    #[prost(int64, tag = "2")]
    pub(super) ctime: i64,
    #[prost(int64, tag = "3")]
    pub(super) mtime: i64,
    #[prost(int64, tag = "4")]
    pub(super) last_time: i64,
}

#[derive(Clone, PartialEq, Message, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PbSettingSyncNeedUploadMessage {
    #[prost(string, tag = "1")]
    pub(super) path: String,
}

#[derive(Clone, PartialEq, Message)]
pub(super) struct FolderSyncCheckRequest {
    #[prost(string, tag = "1")]
    pub(super) path: String,
    #[prost(string, tag = "2")]
    pub(super) path_hash: String,
    #[prost(int64, tag = "3")]
    pub(super) mtime: i64,
}

#[derive(Clone, PartialEq, Message)]
pub(super) struct FolderSyncDelFolder {
    #[prost(string, tag = "1")]
    pub(super) path: String,
    #[prost(string, tag = "2")]
    pub(super) path_hash: String,
}

#[derive(Clone, PartialEq, Message)]
pub(super) struct PbFolderSyncRequest {
    #[prost(string, tag = "1")]
    pub(super) context: String,
    #[prost(string, tag = "2")]
    pub(super) vault: String,
    #[prost(int64, tag = "3")]
    pub(super) last_time: i64,
    #[prost(message, repeated, tag = "4")]
    pub(super) folders: Vec<FolderSyncCheckRequest>,
    #[prost(message, repeated, tag = "5")]
    pub(super) del_folders: Vec<FolderSyncDelFolder>,
    #[prost(message, repeated, tag = "6")]
    pub(super) missing_folders: Vec<FolderSyncDelFolder>,
}

#[derive(Clone, PartialEq, Message)]
pub(super) struct PbFolderCreateRequest {
    #[prost(string, tag = "1")]
    pub(super) vault: String,
    #[prost(string, tag = "2")]
    pub(super) path: String,
    #[prost(string, tag = "3")]
    pub(super) path_hash: String,
}

#[derive(Clone, PartialEq, Message)]
pub(super) struct PbFolderDeleteRequest {
    #[prost(string, tag = "1")]
    pub(super) vault: String,
    #[prost(string, tag = "2")]
    pub(super) path: String,
    #[prost(string, tag = "3")]
    pub(super) path_hash: String,
}

#[derive(Clone, PartialEq, Message)]
pub(super) struct PbFolderRenameRequest {
    #[prost(string, tag = "1")]
    pub(super) vault: String,
    #[prost(string, tag = "2")]
    pub(super) path: String,
    #[prost(string, tag = "3")]
    pub(super) path_hash: String,
    #[prost(string, tag = "4")]
    pub(super) old_path: String,
    #[prost(string, tag = "5")]
    pub(super) old_path_hash: String,
}

#[derive(Clone, PartialEq, Message, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PbFolderSyncModifyMessage {
    #[prost(string, tag = "1")]
    pub(super) path: String,
    #[prost(string, tag = "2")]
    pub(super) path_hash: String,
    #[prost(int64, tag = "3")]
    pub(super) ctime: i64,
    #[prost(int64, tag = "4")]
    pub(super) mtime: i64,
    #[prost(int64, tag = "5")]
    pub(super) last_time: i64,
}

#[derive(Clone, PartialEq, Message, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PbFolderSyncDeleteMessage {
    #[prost(string, tag = "1")]
    pub(super) path: String,
    #[prost(string, tag = "2")]
    pub(super) path_hash: String,
    #[prost(int64, tag = "3")]
    pub(super) ctime: i64,
    #[prost(int64, tag = "4")]
    pub(super) mtime: i64,
    #[prost(int64, tag = "5")]
    pub(super) last_time: i64,
}

#[derive(Clone, PartialEq, Message, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PbFolderSyncRenameMessage {
    #[prost(string, tag = "1")]
    pub(super) path: String,
    #[prost(string, tag = "2")]
    pub(super) path_hash: String,
    #[prost(int64, tag = "3")]
    pub(super) ctime: i64,
    #[prost(int64, tag = "4")]
    pub(super) mtime: i64,
    #[prost(string, tag = "5")]
    pub(super) old_path: String,
    #[prost(string, tag = "6")]
    pub(super) old_path_hash: String,
    #[prost(int64, tag = "7")]
    pub(super) last_time: i64,
}

#[derive(Clone, PartialEq, Message, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PbFolderSyncEndMessage {
    #[prost(int64, tag = "1")]
    pub(super) last_time: i64,
    #[prost(int64, tag = "2")]
    pub(super) need_modify_count: i64,
    #[prost(int64, tag = "3")]
    pub(super) need_delete_count: i64,
}
