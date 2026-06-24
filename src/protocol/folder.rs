use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderSyncCheckRequest {
    pub path: String,
    pub path_hash: String,
    pub mtime: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderSyncDelFolder {
    pub path: String,
    pub path_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderSyncRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    pub vault: String,
    pub last_time: i64,
    #[serde(default)]
    pub folders: Vec<FolderSyncCheckRequest>,
    #[serde(default)]
    pub del_folders: Vec<FolderSyncDelFolder>,
    #[serde(default)]
    pub missing_folders: Vec<FolderSyncDelFolder>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderCreateRequest {
    pub vault: String,
    pub path: String,
    pub path_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderDeleteRequest {
    pub vault: String,
    pub path: String,
    pub path_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderRenameRequest {
    pub vault: String,
    pub path: String,
    pub path_hash: String,
    pub old_path: String,
    pub old_path_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderSyncModifyMessage {
    pub path: String,
    pub path_hash: String,
    pub ctime: i64,
    pub mtime: i64,
    pub last_time: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderSyncDeleteMessage {
    pub path: String,
    pub path_hash: String,
    pub ctime: i64,
    pub mtime: i64,
    pub last_time: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderSyncRenameMessage {
    pub path: String,
    pub path_hash: String,
    pub ctime: i64,
    pub mtime: i64,
    pub old_path: String,
    pub old_path_hash: String,
    pub last_time: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderSyncEndMessage {
    pub last_time: i64,
    pub need_modify_count: i64,
    pub need_delete_count: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderModifyAckMessage {
    pub last_time: i64,
    pub path: String,
    pub path_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderRenameAckMessage {
    pub last_time: i64,
    pub path: String,
    pub path_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderDeleteAckMessage {
    pub last_time: i64,
    pub path: String,
    pub path_hash: String,
}
