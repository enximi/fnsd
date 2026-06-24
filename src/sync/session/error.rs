pub type Result<T> = std::result::Result<T, SyncSessionError>;

#[derive(Debug, thiserror::Error)]
pub enum SyncSessionError {
    #[error(transparent)]
    Config(#[from] crate::config::ConfigError),
    #[error(transparent)]
    Core(#[from] crate::core::CoreError),
    #[error(transparent)]
    FileTransfer(#[from] crate::sync::transfer::FileTransferError),
    #[error(transparent)]
    LocalStore(#[from] crate::store::LocalStoreError),
    #[error(transparent)]
    Plan(#[from] crate::sync::plan::PlanError),
    #[error(transparent)]
    SyncApply(#[from] crate::sync::apply::SyncApplyError),
    #[error(transparent)]
    SyncEngine(#[from] crate::sync::engine::SyncEngineError),
    #[error(transparent)]
    VaultFs(#[from] crate::vault::fs::VaultFsError),
    #[error(transparent)]
    WsClient(#[from] crate::ws::WsClientError),
    #[error("websocket closed")]
    WebSocketClosed,
}
