use crate::core::{ContentHash, ResourceKind, VaultPath};

pub type Result<T> = std::result::Result<T, SyncApplyError>;

#[derive(Debug, thiserror::Error)]
pub enum SyncApplyError {
    #[error(transparent)]
    Core(#[from] crate::core::CoreError),
    #[error(transparent)]
    LocalStore(#[from] crate::store::LocalStoreError),
    #[error(transparent)]
    Plan(#[from] crate::sync::plan::PlanError),
    #[error(transparent)]
    Protocol(#[from] crate::protocol::ProtocolError),
    #[error(transparent)]
    VaultFs(#[from] crate::vault::fs::VaultFsError),
    #[error("authorization rejected: {0}")]
    AuthorizationRejected(String),
    #[error("{kind:?} content hash mismatch for {path}: expected {expected}, got {actual}")]
    ContentHashMismatch {
        kind: ResourceKind,
        path: VaultPath,
        expected: ContentHash,
        actual: ContentHash,
    },
}
