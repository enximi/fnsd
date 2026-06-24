use std::collections::{BTreeMap, VecDeque};
use std::time::{Duration, Instant};

use crate::core::{ResourceKind, VaultPath};
use crate::sync::plan::{FileDownload, FileUpload, RemoteFile};
use crate::sync::transfer::DownloadSession;
use tracing::{debug, warn};

use crate::sync::engine::{Result, SyncEngineError};

#[derive(Debug)]
pub(crate) struct TransferState {
    options: TransferOptions,
    active: BTreeMap<TransferKey, ActiveTransfer>,
    queued: VecDeque<QueuedTransfer>,
    downloads: BTreeMap<String, DownloadEntry>,
    pending_downloads: BTreeMap<String, TransferKey>,
}

impl TransferState {
    pub fn new(options: TransferOptions) -> Self {
        Self {
            options,
            active: BTreeMap::new(),
            queued: VecDeque::new(),
            downloads: BTreeMap::new(),
            pending_downloads: BTreeMap::new(),
        }
    }

    pub fn enqueue(&mut self, transfer: QueuedTransfer) {
        debug!(?transfer, "enqueue transfer");
        self.queued.push_back(transfer);
    }

    pub fn next_ready(&mut self) -> Result<Option<QueuedTransfer>> {
        self.ensure_not_timed_out()?;

        if !self.has_capacity() {
            return Ok(None);
        }

        let index = self
            .queued
            .iter()
            .enumerate()
            .max_by_key(|(_, transfer)| transfer.priority())
            .map(|(index, _)| index);
        let Some(index) = index else {
            return Ok(None);
        };
        let Some(transfer) = self.queued.remove(index) else {
            return Ok(None);
        };
        self.active.insert(
            transfer.key(),
            ActiveTransfer {
                started_at: Instant::now(),
            },
        );
        Ok(Some(transfer))
    }

    pub fn insert_download(&mut self, session: DownloadSession) {
        let path = session.path().to_string();
        if let Some(request_key) = self.pending_downloads.remove(&path) {
            self.active.remove(&request_key);
        }
        let session_key = TransferKey::download_session(session.session_id().to_string());
        self.active.insert(
            session_key,
            ActiveTransfer {
                started_at: Instant::now(),
            },
        );
        self.downloads.insert(
            session.session_id().to_string(),
            DownloadEntry {
                session,
                started_at: Instant::now(),
            },
        );
    }

    pub fn track_download_request(&mut self, file: &RemoteFile, key: TransferKey) {
        self.pending_downloads.insert(file.path.to_string(), key);
    }

    pub fn download_mut(&mut self, session_id: &str) -> Option<&mut DownloadSession> {
        self.downloads
            .get_mut(session_id)
            .map(|entry| &mut entry.session)
    }

    pub fn take_download(&mut self, session_id: &str) -> Option<DownloadSession> {
        self.downloads.remove(session_id).map(|entry| entry.session)
    }

    pub fn finish(&mut self, key: TransferKey) {
        self.active.remove(&key);
    }

    pub fn has_pending_work(&mut self) -> Result<bool> {
        self.ensure_not_timed_out()?;
        Ok(!self.queued.is_empty() || !self.active.is_empty() || !self.downloads.is_empty())
    }

    fn has_capacity(&self) -> bool {
        !self.options.concurrency_enabled
            || self.active.len() < self.options.max_concurrent_transfers
    }

    fn ensure_not_timed_out(&self) -> Result<()> {
        if self.options.timeout.is_zero() {
            return Ok(());
        }

        let now = Instant::now();
        let timeout = self.options.timeout;
        if let Some((key, _)) = self
            .active
            .iter()
            .find(|(_, transfer)| now.duration_since(transfer.started_at) > timeout)
        {
            warn!(transfer = %key.description(), "transfer timed out");
            return Err(SyncEngineError::TransferTimeout(key.description()));
        }

        if let Some((session_id, _)) = self
            .downloads
            .iter()
            .find(|(_, transfer)| now.duration_since(transfer.started_at) > timeout)
        {
            warn!(session_id = %session_id, "download session timed out");
            return Err(SyncEngineError::TransferTimeout(format!(
                "download session {session_id}"
            )));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransferOptions {
    pub concurrency_enabled: bool,
    pub max_concurrent_transfers: usize,
    pub timeout: Duration,
}

impl TransferOptions {
    pub fn new(
        concurrency_enabled: bool,
        max_concurrent_transfers: usize,
        timeout_seconds: u64,
    ) -> Result<Self> {
        if concurrency_enabled && max_concurrent_transfers == 0 {
            return Err(SyncEngineError::InvalidTransferConcurrency);
        }

        Ok(Self {
            concurrency_enabled,
            max_concurrent_transfers: max_concurrent_transfers.max(1),
            timeout: Duration::from_secs(timeout_seconds),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum QueuedTransfer {
    NoteUpload(VaultPath),
    FileUpload(FileUpload),
    FileDownloadRequest(RemoteFile),
    FileDownloadSession(FileDownload),
    SettingUpload(VaultPath),
}

impl QueuedTransfer {
    pub fn key(&self) -> TransferKey {
        match self {
            Self::NoteUpload(path) => TransferKey::resource(ResourceKind::Note, path),
            Self::FileUpload(upload) => TransferKey::resource(ResourceKind::File, &upload.path),
            Self::FileDownloadRequest(file) => {
                TransferKey::download_request(file.path_hash.to_string())
            }
            Self::FileDownloadSession(download) => {
                TransferKey::download_session(download.session_id.clone())
            }
            Self::SettingUpload(path) => TransferKey::resource(ResourceKind::Setting, path),
        }
    }

    fn priority(&self) -> i8 {
        match self {
            Self::NoteUpload(_) | Self::FileUpload(_) | Self::SettingUpload(_) => 10,
            Self::FileDownloadRequest(_) | Self::FileDownloadSession(_) => -10,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum TransferKey {
    Resource(ResourceKind, VaultPath),
    DownloadRequest(String),
    DownloadSession(String),
}

impl TransferKey {
    fn resource(kind: ResourceKind, path: &VaultPath) -> Self {
        Self::Resource(kind, path.clone())
    }

    fn download_request(path_hash: String) -> Self {
        Self::DownloadRequest(path_hash)
    }

    fn download_session(session_id: String) -> Self {
        Self::DownloadSession(session_id)
    }

    fn description(&self) -> String {
        match self {
            Self::Resource(kind, path) => format!("{kind:?} {path}"),
            Self::DownloadRequest(path_hash) => format!("download request {path_hash}"),
            Self::DownloadSession(session_id) => format!("download session {session_id}"),
        }
    }
}

#[derive(Debug)]
struct ActiveTransfer {
    started_at: Instant,
}

#[derive(Debug)]
struct DownloadEntry {
    session: DownloadSession,
    started_at: Instant,
}
