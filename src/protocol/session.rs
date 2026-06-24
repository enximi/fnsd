use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientInfoMessage {
    pub name: String,
    pub version: String,
    #[serde(rename = "type")]
    pub client_type: String,
    #[serde(default)]
    pub is_desktop: bool,
    #[serde(default)]
    pub is_mobile: bool,
    #[serde(default)]
    pub is_phone: bool,
    #[serde(default)]
    pub is_tablet: bool,
    #[serde(default)]
    pub is_mac_os: bool,
    #[serde(default)]
    pub is_win: bool,
    #[serde(default)]
    pub is_linux: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offline_sync_strategy: Option<OfflineSyncStrategy>,
    #[serde(default)]
    pub protobuf: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OfflineSyncStrategy {
    #[serde(rename = "newTimeMerge")]
    NewTimeMerge,
    #[serde(rename = "ignoreTimeMerge")]
    IgnoreTimeMerge,
}
