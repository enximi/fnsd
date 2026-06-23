pub type Result<T> = std::result::Result<T, SyncEngineError>;

#[derive(Debug, thiserror::Error)]
pub enum SyncEngineError {
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
    VaultFs(#[from] fns_vault_fs::VaultFsError),
    #[error(transparent)]
    WsClient(#[from] fns_ws_client::WsClientError),
    #[error("blocking task failed: {0}")]
    BlockingTask(String),
    #[error("checkpoint io failed for {path}: {source}")]
    CheckpointIo {
        path: std::path::PathBuf,
        source: std::io::Error,
    },
    #[error("max concurrent transfers must be greater than 0 when transfer concurrency is enabled")]
    InvalidTransferConcurrency,
    #[error("transfer timed out: {0}")]
    TransferTimeout(String),
    #[error(
        "upload checkpoint chunk index {index} is out of range for {path}: total chunks {total}"
    )]
    UploadCheckpointOutOfRange {
        path: fns_core::VaultPath,
        index: u32,
        total: usize,
    },
    #[error("websocket closed before sync completed")]
    WebSocketClosed,
}
