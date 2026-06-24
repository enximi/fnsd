use std::path::PathBuf;
use std::sync::mpsc::{RecvError, TryRecvError};

pub type Result<T> = std::result::Result<T, VaultWatchError>;

#[derive(Debug, thiserror::Error)]
pub enum VaultWatchError {
    #[error(transparent)]
    Core(#[from] crate::core::CoreError),
    #[error("filesystem error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error(transparent)]
    Notify(#[from] notify::Error),
    #[error("watch event channel closed")]
    ChannelClosed,
}

impl From<RecvError> for VaultWatchError {
    fn from(_: RecvError) -> Self {
        Self::ChannelClosed
    }
}

impl From<TryRecvError> for VaultWatchError {
    fn from(value: TryRecvError) -> Self {
        match value {
            TryRecvError::Empty => Self::ChannelClosed,
            TryRecvError::Disconnected => Self::ChannelClosed,
        }
    }
}

pub(crate) fn io(path: impl Into<PathBuf>, source: std::io::Error) -> VaultWatchError {
    VaultWatchError::Io {
        path: path.into(),
        source,
    }
}
