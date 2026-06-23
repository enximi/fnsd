use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, LocalStoreError>;

#[derive(Debug, thiserror::Error)]
pub enum LocalStoreError {
    #[error(transparent)]
    Core(#[from] fns_core::CoreError),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
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
