use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, LocalStoreError>;

#[derive(Debug, thiserror::Error)]
pub enum LocalStoreError {
    #[error(transparent)]
    Core(#[from] crate::core::CoreError),
    #[error(transparent)]
    FileTransfer(#[from] crate::sync::transfer::FileTransferError),
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
    #[error("store value {name} is too large: {value}")]
    NumberTooLarge { name: &'static str, value: u64 },
    #[error("store io error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

pub(crate) fn io(path: impl Into<PathBuf>, source: std::io::Error) -> LocalStoreError {
    LocalStoreError::Io {
        path: path.into(),
        source,
    }
}
