use std::collections::BTreeMap;

use fns_core::{ContentHash, RemoteMillis};
use serde::{Deserialize, Serialize};

use crate::Result;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalStoreState {
    #[serde(default = "current_version")]
    pub version: u32,
    #[serde(default)]
    pub sync_times: SyncTimes,
    #[serde(default)]
    pub hashes: ResourceHashes,
    #[serde(default)]
    pub pending: crate::PendingState,
}

impl Default for LocalStoreState {
    fn default() -> Self {
        Self {
            version: current_version(),
            sync_times: SyncTimes::default(),
            hashes: ResourceHashes::default(),
            pending: crate::PendingState::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncTimes {
    #[serde(default)]
    pub note: i64,
    #[serde(default)]
    pub file: i64,
    #[serde(default)]
    pub folder: i64,
    #[serde(default)]
    pub setting: i64,
}

impl SyncTimes {
    pub fn get(self, kind: fns_core::ResourceKind) -> Result<RemoteMillis> {
        Ok(RemoteMillis::new(match kind {
            fns_core::ResourceKind::Note => self.note,
            fns_core::ResourceKind::File => self.file,
            fns_core::ResourceKind::Folder => self.folder,
            fns_core::ResourceKind::Setting => self.setting,
        })?)
    }

    pub fn set(&mut self, kind: fns_core::ResourceKind, value: RemoteMillis) {
        match kind {
            fns_core::ResourceKind::Note => self.note = value.as_i64(),
            fns_core::ResourceKind::File => self.file = value.as_i64(),
            fns_core::ResourceKind::Folder => self.folder = value.as_i64(),
            fns_core::ResourceKind::Setting => self.setting = value.as_i64(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceHashes {
    #[serde(default)]
    pub notes: BTreeMap<String, HashEntry>,
    #[serde(default)]
    pub files: BTreeMap<String, HashEntry>,
    #[serde(default)]
    pub folders: BTreeMap<String, HashEntry>,
    #[serde(default)]
    pub settings: BTreeMap<String, HashEntry>,
}

impl ResourceHashes {
    pub(crate) fn by_kind(&self, kind: fns_core::ResourceKind) -> &BTreeMap<String, HashEntry> {
        match kind {
            fns_core::ResourceKind::Note => &self.notes,
            fns_core::ResourceKind::File => &self.files,
            fns_core::ResourceKind::Folder => &self.folders,
            fns_core::ResourceKind::Setting => &self.settings,
        }
    }

    pub(crate) fn by_kind_mut(
        &mut self,
        kind: fns_core::ResourceKind,
    ) -> &mut BTreeMap<String, HashEntry> {
        match kind {
            fns_core::ResourceKind::Note => &mut self.notes,
            fns_core::ResourceKind::File => &mut self.files,
            fns_core::ResourceKind::Folder => &mut self.folders,
            fns_core::ResourceKind::Setting => &mut self.settings,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HashEntry {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,
    #[serde(default)]
    pub mtime: i64,
    #[serde(default)]
    pub size: u64,
}

impl HashEntry {
    pub fn new(content_hash: Option<ContentHash>, mtime: RemoteMillis, size: u64) -> Self {
        Self {
            content_hash: content_hash.map(ContentHash::into_string),
            mtime: mtime.as_i64(),
            size,
        }
    }

    pub fn content_hash(&self) -> Result<Option<ContentHash>> {
        Ok(self
            .content_hash
            .as_ref()
            .map(ContentHash::new)
            .transpose()?)
    }

    pub fn mtime(&self) -> Result<RemoteMillis> {
        Ok(RemoteMillis::new(self.mtime)?)
    }
}

fn current_version() -> u32 {
    1
}
