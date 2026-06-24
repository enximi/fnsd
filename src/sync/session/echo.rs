use std::time::Duration;

use crate::core::{ResourceKind, VaultPath};
use crate::sync::apply::EventOutcome;
use crate::vault::watch::VaultWatchEvent;
use tokio::time::Instant;

#[derive(Debug, Default)]
pub(crate) struct RemoteEchoes {
    renames: Vec<EchoRename>,
}

impl RemoteEchoes {
    pub(crate) fn record_outcome(&mut self, outcome: &EventOutcome) {
        match outcome {
            EventOutcome::RemoteWrite { .. }
            | EventOutcome::RemoteDelete { .. }
            | EventOutcome::RemoteMtimeUpdate { .. } => {}
            EventOutcome::RemoteRename {
                kind,
                old_path,
                new_path,
            } => {
                self.record_rename(*kind, old_path.clone(), new_path.clone());
            }
            EventOutcome::AuthorizationAccepted
            | EventOutcome::Ack { .. }
            | EventOutcome::SyncEnd { .. }
            | EventOutcome::NoteUploadRequested(_)
            | EventOutcome::FileUploadRequested(_)
            | EventOutcome::FileDownloadRequested(_)
            | EventOutcome::FileDownloadSessionReady(_)
            | EventOutcome::SettingUploadRequested(_)
            | EventOutcome::Ignored => {}
        }
    }

    fn record_rename(&mut self, kind: ResourceKind, old_path: VaultPath, new_path: VaultPath) {
        self.renames.push(EchoRename {
            kind,
            old_path,
            new_path,
            old_seen: false,
            new_seen: false,
            expires_at: echo_expires_at(),
        });
    }

    pub(crate) fn consume(&mut self, event: &VaultWatchEvent) -> bool {
        self.prune_expired();

        match event {
            VaultWatchEvent::Changed { path } => self.consume_rename_changed(path),
            VaultWatchEvent::RenameFrom { path } => self.consume_rename_from(path),
            VaultWatchEvent::RenameTo { path } => self.consume_rename_to(path),
            VaultWatchEvent::Renamed { old_path, new_path } => {
                self.consume_rename(old_path, new_path)
            }
            VaultWatchEvent::RescanNeeded => false,
        }
    }

    fn consume_rename_changed(&mut self, path: &VaultPath) -> bool {
        let Some(index) = self
            .renames
            .iter_mut()
            .position(|echo| echo.mark_changed(path))
        else {
            return false;
        };
        if self.renames[index].is_complete() {
            self.renames.remove(index);
        }
        true
    }

    fn consume_rename_from(&mut self, path: &VaultPath) -> bool {
        self.consume_rename_side(path, RenameEchoSide::Old)
    }

    fn consume_rename_to(&mut self, path: &VaultPath) -> bool {
        self.consume_rename_side(path, RenameEchoSide::New)
    }

    fn consume_rename_side(&mut self, path: &VaultPath, side: RenameEchoSide) -> bool {
        let Some(index) = self
            .renames
            .iter_mut()
            .position(|echo| echo.mark_side(path, side))
        else {
            return false;
        };
        if self.renames[index].is_complete() {
            self.renames.remove(index);
        }
        true
    }

    fn consume_rename(&mut self, old_path: &VaultPath, new_path: &VaultPath) -> bool {
        let Some(index) = self
            .renames
            .iter()
            .position(|echo| echo.matches_rename(old_path, new_path))
        else {
            return false;
        };
        self.renames.remove(index);
        true
    }

    fn prune_expired(&mut self) {
        let now = Instant::now();
        self.renames.retain(|echo| echo.expires_at > now);
    }
}

#[derive(Debug)]
struct EchoRename {
    kind: ResourceKind,
    old_path: VaultPath,
    new_path: VaultPath,
    old_seen: bool,
    new_seen: bool,
    expires_at: Instant,
}

impl EchoRename {
    fn mark_changed(&mut self, path: &VaultPath) -> bool {
        let old_matches = paths_match(self.kind, &self.old_path, path);
        let new_matches = paths_match(self.kind, &self.new_path, path);
        self.old_seen |= old_matches;
        self.new_seen |= new_matches;
        old_matches || new_matches
    }

    fn mark_side(&mut self, path: &VaultPath, side: RenameEchoSide) -> bool {
        let matches = match side {
            RenameEchoSide::Old => paths_match(self.kind, &self.old_path, path),
            RenameEchoSide::New => paths_match(self.kind, &self.new_path, path),
        };
        if !matches {
            return false;
        }

        match side {
            RenameEchoSide::Old => self.old_seen = true,
            RenameEchoSide::New => self.new_seen = true,
        }
        true
    }

    fn matches_rename(&self, old_path: &VaultPath, new_path: &VaultPath) -> bool {
        paths_match(self.kind, &self.old_path, old_path)
            && paths_match(self.kind, &self.new_path, new_path)
    }

    fn is_complete(&self) -> bool {
        self.old_seen && self.new_seen
    }
}

#[derive(Debug, Clone, Copy)]
enum RenameEchoSide {
    Old,
    New,
}

fn paths_match(kind: ResourceKind, expected: &VaultPath, actual: &VaultPath) -> bool {
    expected == actual || (kind == ResourceKind::Folder && is_child_path(expected, actual))
}

fn is_child_path(parent: &VaultPath, path: &VaultPath) -> bool {
    path.as_str()
        .strip_prefix(parent.as_str())
        .is_some_and(|suffix| suffix.starts_with('/'))
}

fn echo_expires_at() -> Instant {
    Instant::now() + Duration::from_secs(10)
}
