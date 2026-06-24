use std::path::PathBuf;

use crate::core::VaultPath;

pub type Result<T> = std::result::Result<T, VaultFsError>;

#[derive(Debug, thiserror::Error)]
pub enum VaultFsError {
    #[error(transparent)]
    Core(#[from] crate::core::CoreError),
    #[error("filesystem error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("vault root is not a directory: {0}")]
    RootNotDirectory(PathBuf),
    #[error("path escapes vault root: {0}")]
    EscapesVault(VaultPath),
    #[error("path is not valid UTF-8 under vault root: {0}")]
    NonUtf8Path(PathBuf),
}

pub(crate) fn io(path: impl Into<PathBuf>, source: std::io::Error) -> VaultFsError {
    VaultFsError::Io {
        path: path.into(),
        source,
    }
}
