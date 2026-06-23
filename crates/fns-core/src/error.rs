pub type Result<T> = std::result::Result<T, CoreError>;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CoreError {
    #[error("vault name is empty")]
    EmptyVaultName,
    #[error("vault name is invalid")]
    InvalidVaultName,
    #[error("vault path is empty")]
    EmptyPath,
    #[error("vault path must be relative")]
    AbsolutePath,
    #[error("vault path cannot contain parent traversal")]
    ParentTraversal,
    #[error("vault path is invalid")]
    InvalidPath,
    #[error("hash value is invalid")]
    InvalidHash,
    #[error("timestamp is invalid")]
    InvalidTimestamp,
}
