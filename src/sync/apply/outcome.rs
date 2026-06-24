use crate::core::{RemoteMillis, ResourceKind, VaultPath};
use crate::sync::plan::{DeletedResource, FileDownload, FileUpload, RemoteFile};

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
            EventOutcome::NoteUploadRequested(_) => self.pending_note_uploads += 1,
            EventOutcome::FileUploadRequested(_) => self.pending_file_uploads += 1,
            EventOutcome::FileDownloadRequested(_) | EventOutcome::FileDownloadSessionReady(_) => {
                self.pending_file_downloads += 1;
            }
            EventOutcome::SettingUploadRequested(_) => self.pending_setting_uploads += 1,
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
        pending_events: usize,
    },
    NoteUploadRequested(DeletedResource),
    FileUploadRequested(FileUpload),
    FileDownloadRequested(RemoteFile),
    FileDownloadSessionReady(FileDownload),
    SettingUploadRequested(VaultPath),
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

pub fn pending_sync_end_events(
    need_upload_count: i64,
    need_modify_count: i64,
    need_sync_mtime_count: i64,
    need_delete_count: i64,
) -> usize {
    [
        need_upload_count,
        need_modify_count,
        need_sync_mtime_count,
        need_delete_count,
    ]
    .into_iter()
    .filter_map(|value| usize::try_from(value).ok())
    .sum()
}
