pub type Result<T> = std::result::Result<T, SyncSessionError>;

#[derive(Debug, thiserror::Error)]
pub enum SyncSessionError {
    #[error(transparent)]
    Config(#[from] fns_config::ConfigError),
    #[error(transparent)]
    Core(#[from] fns_core::CoreError),
    #[error(transparent)]
    FileTransfer(#[from] fns_file_transfer::FileTransferError),
    #[error(transparent)]
    LocalStore(#[from] fns_local_store::LocalStoreError),
    #[error(transparent)]
    Plan(#[from] fns_sync_plan::PlanError),
    #[error(transparent)]
    SyncApply(#[from] fns_sync_apply::SyncApplyError),
    #[error(transparent)]
    SyncEngine(#[from] fns_sync_engine::SyncEngineError),
    #[error(transparent)]
    VaultFs(#[from] fns_vault_fs::VaultFsError),
    #[error(transparent)]
    WsClient(#[from] fns_ws_client::WsClientError),
    #[error("websocket closed")]
    WebSocketClosed,
}
