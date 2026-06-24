use crate::core::VaultPath;

pub type Result<T> = std::result::Result<T, PlanError>;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PlanError {
    #[error(transparent)]
    Core(#[from] crate::core::CoreError),
    #[error("file size is too large for protocol field: {path} has {size} bytes")]
    FileSizeTooLarge { path: VaultPath, size: u64 },
}
