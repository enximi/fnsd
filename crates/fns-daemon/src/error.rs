pub type Result<T> = std::result::Result<T, DaemonError>;

#[derive(Debug, thiserror::Error)]
pub enum DaemonError {
    #[error(transparent)]
    Config(#[from] fns_config::ConfigError),
    #[error(transparent)]
    SyncSession(#[from] fns_sync_session::SyncSessionError),
    #[error(transparent)]
    Watch(#[from] fns_vault_watch::VaultWatchError),
    #[error("watch task stopped")]
    WatchTaskStopped,
}
