use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileSyncRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    pub vault: String,
    pub last_time: i64,
    #[serde(default)]
    pub files: Vec<FileSyncCheckRequest>,
    #[serde(default)]
    pub del_files: Vec<FileSyncDelFile>,
    #[serde(default)]
    pub missing_files: Vec<FileSyncDelFile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileSyncCheckRequest {
    pub path: String,
    pub path_hash: String,
    pub content_hash: String,
    pub size: i64,
    pub mtime: i64,
    pub ctime: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileSyncDelFile {
    pub path: String,
    pub path_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileUploadCheckRequest {
    pub vault: String,
    pub path: String,
    pub path_hash: String,
    pub content_hash: String,
    pub size: i64,
    pub ctime: i64,
    pub mtime: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileDeleteRequest {
    pub vault: String,
    pub path: String,
    pub path_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileRenameRequest {
    pub vault: String,
    pub path: String,
    pub path_hash: String,
    pub old_path: String,
    pub old_path_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileChunkDownloadRequest {
    pub vault: String,
    pub path: String,
    pub path_hash: String,
    pub session_id: String,
    pub chunk_index: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileGetRequest {
    pub vault: String,
    pub path: String,
    pub path_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileSyncModifyMessage {
    pub path: String,
    pub path_hash: String,
    pub content_hash: String,
    pub size: i64,
    pub ctime: i64,
    pub mtime: i64,
    pub last_time: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileSyncDeleteMessage {
    pub path: String,
    pub path_hash: String,
    pub ctime: i64,
    pub mtime: i64,
    pub size: i64,
    pub last_time: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileSyncRenameMessage {
    pub path: String,
    pub path_hash: String,
    pub content_hash: String,
    pub ctime: i64,
    pub mtime: i64,
    pub size: i64,
    pub last_time: i64,
    pub old_path: String,
    pub old_path_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileSyncMtimeMessage {
    pub path: String,
    pub ctime: i64,
    pub mtime: i64,
    pub last_time: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileSyncEndMessage {
    pub last_time: i64,
    pub need_upload_count: i64,
    pub need_modify_count: i64,
    pub need_sync_mtime_count: i64,
    pub need_delete_count: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileSyncUploadMessage {
    pub path: String,
    pub path_hash: String,
    pub session_id: String,
    pub chunk_size: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileSyncDownloadMessage {
    pub path: String,
    pub content_hash: String,
    pub ctime: i64,
    pub mtime: i64,
    pub session_id: String,
    pub chunk_size: i64,
    pub total_chunks: i64,
    pub size: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileRenameAckMessage {
    pub last_time: i64,
    pub path: String,
    pub path_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileUploadAckMessage {
    pub last_time: i64,
    pub path: String,
    pub path_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileDeleteAckMessage {
    pub last_time: i64,
    pub path: String,
    pub path_hash: String,
}
