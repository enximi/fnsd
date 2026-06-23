pub type Result<T> = std::result::Result<T, SyncApplyError>;

#[derive(Debug, thiserror::Error)]
pub enum SyncApplyError {
    #[error(transparent)]
    Core(#[from] fns_core::CoreError),
    #[error(transparent)]
    LocalStore(#[from] fns_local_store::LocalStoreError),
    #[error(transparent)]
    Plan(#[from] fns_sync_plan::PlanError),
    #[error(transparent)]
    Protocol(#[from] fns_protocol::ProtocolError),
    #[error(transparent)]
    VaultFs(#[from] fns_vault_fs::VaultFsError),
    #[error("authorization rejected: {0}")]
    AuthorizationRejected(String),
}
