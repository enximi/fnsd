pub type Result<T> = std::result::Result<T, SyncEngineError>;

#[derive(Debug, thiserror::Error)]
pub enum SyncEngineError {
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
    VaultFs(#[from] crate::vault::fs::VaultFsError),
    #[error(transparent)]
    WsClient(#[from] crate::ws::WsClientError),
    #[error("blocking task failed: {0}")]
    BlockingTask(String),
    #[error("max concurrent transfers must be greater than 0 when transfer concurrency is enabled")]
    InvalidTransferConcurrency,
    #[error("transfer timed out: {0}")]
    TransferTimeout(String),
    #[error(
        "upload checkpoint chunk index {index} is out of range for {path}: total chunks {total}"
    )]
    UploadCheckpointOutOfRange {
        path: crate::core::VaultPath,
        index: u32,
        total: usize,
    },
    #[error("websocket closed before sync completed")]
    WebSocketClosed,
}
