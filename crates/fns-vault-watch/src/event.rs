use fns_core::VaultPath;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VaultWatchEvent {
    Changed { path: VaultPath },
    RescanNeeded,
}
