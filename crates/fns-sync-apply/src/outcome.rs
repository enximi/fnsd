use fns_core::{RemoteMillis, ResourceKind, VaultPath};
use fns_sync_plan::{DeletedResource, FileDownload, FileUpload, RemoteFile};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EventApplySummary {
    pub text_events: usize,
    pub remote_writes: usize,
    pub remote_deletes: usize,
    pub remote_renames: usize,
    pub remote_mtime_updates: usize,
    pub acks: usize,
    pub sync_ends: usize,
    pub pending_note_uploads: usize,
    pub pending_file_uploads: usize,
    pub pending_file_downloads: usize,
    pub pending_setting_uploads: usize,
}

impl EventApplySummary {
    pub fn add(&mut self, outcome: &EventOutcome) {
        self.text_events += 1;

        match outcome {
            EventOutcome::RemoteWrite { .. } => self.remote_writes += 1,
            EventOutcome::RemoteDelete { .. } => self.remote_deletes += 1,
            EventOutcome::RemoteRename { .. } => self.remote_renames += 1,
            EventOutcome::RemoteMtimeUpdate { .. } => self.remote_mtime_updates += 1,
            EventOutcome::Ack { .. } => self.acks += 1,
            EventOutcome::SyncEnd { .. } => self.sync_ends += 1,
            EventOutcome::NeedNoteUpload(_) => self.pending_note_uploads += 1,
            EventOutcome::NeedFileUpload(_) => self.pending_file_uploads += 1,
            EventOutcome::NeedFileDownload(_) | EventOutcome::NeedFileDownloadSession(_) => {
                self.pending_file_downloads += 1;
            }
            EventOutcome::NeedSettingUpload(_) => self.pending_setting_uploads += 1,
            EventOutcome::AuthorizationAccepted | EventOutcome::Ignored => {}
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventOutcome {
    AuthorizationAccepted,
    RemoteWrite {
        kind: ResourceKind,
        path: VaultPath,
    },
    RemoteDelete {
        kind: ResourceKind,
        path: VaultPath,
    },
    RemoteRename {
        kind: ResourceKind,
        old_path: VaultPath,
        new_path: VaultPath,
    },
    RemoteMtimeUpdate {
        kind: ResourceKind,
        path: VaultPath,
    },
    Ack {
        kind: ResourceKind,
        path: VaultPath,
    },
    SyncEnd {
        kind: ResourceKind,
        last_time: RemoteMillis,
    },
    NeedNoteUpload(DeletedResource),
    NeedFileUpload(FileUpload),
    NeedFileDownload(RemoteFile),
    NeedFileDownloadSession(FileDownload),
    NeedSettingUpload(VaultPath),
    Ignored,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SyncEndTracker {
    note: bool,
    file: bool,
    folder: bool,
    setting: bool,
}

impl SyncEndTracker {
    pub fn mark(&mut self, kind: ResourceKind) {
        match kind {
            ResourceKind::Note => self.note = true,
            ResourceKind::File => self.file = true,
            ResourceKind::Folder => self.folder = true,
            ResourceKind::Setting => self.setting = true,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.note && self.file && self.folder && self.setting
    }
}
