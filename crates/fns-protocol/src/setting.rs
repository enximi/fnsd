use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingSyncRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    pub vault: String,
    pub last_time: i64,
    #[serde(default)]
    pub settings: Vec<SettingSyncCheckRequest>,
    #[serde(default)]
    pub del_settings: Vec<SettingSyncDelSetting>,
    #[serde(default)]
    pub missing_settings: Vec<SettingSyncDelSetting>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingSyncCheckRequest {
    pub path: String,
    pub path_hash: String,
    pub content_hash: String,
    pub mtime: i64,
    pub ctime: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingSyncDelSetting {
    pub path: String,
    pub path_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingModifyOrCreateRequest {
    pub vault: String,
    pub path: String,
    pub path_hash: String,
    pub content: String,
    pub content_hash: String,
    pub ctime: i64,
    pub mtime: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingUpdateCheckRequest {
    pub vault: String,
    pub path: String,
    pub path_hash: String,
    pub content_hash: String,
    pub ctime: i64,
    pub mtime: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingDeleteRequest {
    pub vault: String,
    pub path: String,
    pub path_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingGetRequest {
    pub vault: String,
    pub path: String,
    pub path_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingClearRequest {
    pub vault: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingSyncModifyMessage {
    pub vault: String,
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
pub struct SettingSyncDeleteMessage {
    pub path: String,
    pub path_hash: String,
    pub ctime: i64,
    pub mtime: i64,
    pub last_time: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingSyncMtimeMessage {
    pub path: String,
    pub ctime: i64,
    pub mtime: i64,
    pub last_time: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingSyncEndMessage {
    pub last_time: i64,
    pub need_upload_count: i64,
    pub need_modify_count: i64,
    pub need_sync_mtime_count: i64,
    pub need_delete_count: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingSyncNeedUploadMessage {
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingModifyAckMessage {
    pub last_time: i64,
    pub path: String,
    pub path_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingDeleteAckMessage {
    pub last_time: i64,
    pub path: String,
    pub path_hash: String,
}
