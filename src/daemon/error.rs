pub type Result<T> = std::result::Result<T, DaemonError>;

#[derive(Debug, thiserror::Error)]
pub enum DaemonError {
    #[error(transparent)]
    Config(#[from] crate::config::ConfigError),
    #[error(transparent)]
    SyncSession(#[from] crate::sync::session::SyncSessionError),
    #[error(transparent)]
    Watch(#[from] crate::vault::watch::VaultWatchError),
    #[error("watch task stopped")]
    WatchTaskStopped,
}
