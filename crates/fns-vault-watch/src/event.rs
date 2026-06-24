use fns_core::VaultPath;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VaultWatchEvent {
    Changed {
        path: VaultPath,
    },
    RenameFrom {
        path: VaultPath,
    },
    RenameTo {
        path: VaultPath,
    },
    Renamed {
        old_path: VaultPath,
        new_path: VaultPath,
    },
    RescanNeeded,
}
