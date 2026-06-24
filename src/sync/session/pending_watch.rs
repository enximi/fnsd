use std::collections::BTreeSet;

use crate::core::VaultPath;
use crate::vault::watch::VaultWatchEvent;

#[derive(Debug, Default)]
pub(crate) struct PendingWatchEvents {
    paths: BTreeSet<VaultPath>,
    rename_from: Vec<VaultPath>,
    rename_to: Vec<VaultPath>,
    renames: Vec<(VaultPath, VaultPath)>,
    rescan_needed: bool,
}

impl PendingWatchEvents {
    pub(crate) fn push(&mut self, event: VaultWatchEvent) {
        match event {
            VaultWatchEvent::Changed { path } => {
                if self
                    .renames
                    .iter()
                    .any(|(old_path, new_path)| old_path == &path || new_path == &path)
                {
                    return;
                }
                self.paths.insert(path);
            }
            VaultWatchEvent::RenameFrom { path } => {
                if self.renames.iter().any(|(old_path, _)| old_path == &path) {
                    return;
                }
                self.paths.remove(&path);
                if let Some(new_path) = pop_front(&mut self.rename_to) {
                    self.push_rename(path, new_path);
                } else {
                    self.rename_from.push(path);
                }
            }
            VaultWatchEvent::RenameTo { path } => {
                if self.renames.iter().any(|(_, new_path)| new_path == &path) {
                    return;
                }
                self.paths.remove(&path);
                if let Some(old_path) = pop_front(&mut self.rename_from) {
                    self.push_rename(old_path, path);
                } else {
                    self.rename_to.push(path);
                }
            }
            VaultWatchEvent::Renamed { old_path, new_path } => {
                self.paths.remove(&old_path);
                self.paths.remove(&new_path);
                self.rename_from.retain(|path| path != &old_path);
                self.rename_to.retain(|path| path != &new_path);
                self.push_rename(old_path, new_path);
            }
            VaultWatchEvent::RescanNeeded => {
                self.rescan_needed = true;
            }
        }
    }

    pub(crate) fn take_all(&mut self) -> Vec<VaultWatchEvent> {
        let mut events = Vec::new();
        if self.rescan_needed {
            events.push(VaultWatchEvent::RescanNeeded);
            self.rescan_needed = false;
        }

        events.extend(
            std::mem::take(&mut self.renames)
                .into_iter()
                .map(|(old_path, new_path)| VaultWatchEvent::Renamed { old_path, new_path }),
        );
        events.extend(
            std::mem::take(&mut self.rename_from)
                .into_iter()
                .map(|path| VaultWatchEvent::Changed { path }),
        );
        events.extend(
            std::mem::take(&mut self.rename_to)
                .into_iter()
                .map(|path| VaultWatchEvent::Changed { path }),
        );
        events.extend(
            std::mem::take(&mut self.paths)
                .into_iter()
                .map(|path| VaultWatchEvent::Changed { path }),
        );
        events
    }

    fn push_rename(&mut self, old_path: VaultPath, new_path: VaultPath) {
        if self.renames.iter().any(|(existing_old, existing_new)| {
            existing_old == &old_path && existing_new == &new_path
        }) {
            return;
        }

        self.renames.push((old_path, new_path));
    }
}

fn pop_front<T>(items: &mut Vec<T>) -> Option<T> {
    if items.is_empty() {
        None
    } else {
        Some(items.remove(0))
    }
}
