use crate::core::{ContentHash, PathHash, RemoteMillis, VaultPath};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ResourceKind {
    Note,
    File,
    Folder,
    Setting,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncBatch<T> {
    pub context: Option<String>,
    pub last_time: RemoteMillis,
    pub items: Vec<T>,
    pub deleted: Vec<DeletedResource>,
    pub missing: Vec<DeletedResource>,
}

impl<T> SyncBatch<T> {
    pub fn new(last_time: RemoteMillis) -> Self {
        Self {
            context: None,
            last_time,
            items: Vec::new(),
            deleted: Vec::new(),
            missing: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextResource {
    pub path: VaultPath,
    pub path_hash: PathHash,
    pub content_hash: ContentHash,
    pub ctime: RemoteMillis,
    pub mtime: RemoteMillis,
}

pub type NoteResource = TextResource;
pub type SettingResource = TextResource;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileResource {
    pub path: VaultPath,
    pub path_hash: PathHash,
    pub content_hash: ContentHash,
    pub size: u64,
    pub ctime: RemoteMillis,
    pub mtime: RemoteMillis,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderResource {
    pub path: VaultPath,
    pub path_hash: PathHash,
    pub mtime: RemoteMillis,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeletedResource {
    pub path: VaultPath,
    pub path_hash: PathHash,
}
