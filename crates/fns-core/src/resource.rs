use crate::VaultPath;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ResourceKind {
    Note,
    File,
    Folder,
    Setting,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResourceId {
    kind: ResourceKind,
    path: VaultPath,
}

impl ResourceId {
    pub fn new(kind: ResourceKind, path: VaultPath) -> Self {
        Self { kind, path }
    }

    pub fn kind(&self) -> ResourceKind {
        self.kind
    }

    pub fn path(&self) -> &VaultPath {
        &self.path
    }

    pub fn into_parts(self) -> (ResourceKind, VaultPath) {
        (self.kind, self.path)
    }
}
