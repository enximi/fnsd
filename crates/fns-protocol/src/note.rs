use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteSyncCheckRequest {
    pub path: String,
    pub path_hash: String,
    pub content_hash: String,
    pub mtime: i64,
    pub ctime: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteSyncDelNote {
    pub path: String,
    pub path_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteSyncRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    pub vault: String,
    pub last_time: i64,
    #[serde(default)]
    pub notes: Vec<NoteSyncCheckRequest>,
    #[serde(default)]
    pub del_notes: Vec<NoteSyncDelNote>,
    #[serde(default)]
    pub missing_notes: Vec<NoteSyncDelNote>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteModifyOrCreateRequest {
    pub vault: String,
    pub path: String,
    pub path_hash: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_hash: Option<String>,
    #[serde(default)]
    pub base_hash_missing: bool,
    pub content: String,
    pub content_hash: String,
    pub ctime: i64,
    pub mtime: i64,
    #[serde(default)]
    pub create_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteUpdateCheckRequest {
    pub vault: String,
    pub path: String,
    pub path_hash: String,
    pub content_hash: String,
    pub ctime: i64,
    pub mtime: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteDeleteRequest {
    pub vault: String,
    pub path: String,
    pub path_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteRenameRequest {
    pub vault: String,
    pub path: String,
    pub path_hash: String,
    pub old_path: String,
    pub old_path_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteGetRequest {
    pub vault: String,
    pub path: String,
    pub path_hash: String,
    #[serde(default)]
    pub is_recycle: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteSyncModifyMessage {
    pub path: String,
    pub path_hash: String,
    pub content: String,
    pub content_hash: String,
    pub ctime: i64,
    pub mtime: i64,
    pub last_time: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteSyncDeleteMessage {
    pub path: String,
    pub path_hash: String,
    pub ctime: i64,
    pub mtime: i64,
    pub size: i64,
    pub last_time: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteSyncRenameMessage {
    pub path: String,
    pub path_hash: String,
    pub content_hash: String,
    pub ctime: i64,
    pub mtime: i64,
    pub size: i64,
    pub old_path: String,
    pub old_path_hash: String,
    pub last_time: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteSyncMtimeMessage {
    pub path: String,
    pub ctime: i64,
    pub mtime: i64,
    pub last_time: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteSyncEndMessage {
    pub last_time: i64,
    pub need_upload_count: i64,
    pub need_modify_count: i64,
    pub need_sync_mtime_count: i64,
    pub need_delete_count: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteSyncNeedPushMessage {
    pub path: String,
    pub path_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteModifyAckMessage {
    pub last_time: i64,
    pub path: String,
    pub path_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteRenameAckMessage {
    pub last_time: i64,
    pub path: String,
    pub path_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteDeleteAckMessage {
    pub last_time: i64,
    pub path: String,
    pub path_hash: String,
}
