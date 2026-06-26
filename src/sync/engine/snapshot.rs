use std::collections::BTreeSet;

use crate::core::{
    ContentHash, DeletedResource, FileResource, FolderResource, NoteResource, RemoteMillis,
    ResourceKind, SettingResource, SyncBatch, TextResource, VaultPath,
};
use crate::hash::path_hash;
use crate::store::LocalStore;
use crate::vault::fs::{VaultScanOptions, VaultSnapshot};

use crate::sync::engine::{MissingPathMode, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SyncBatches {
    pub notes: SyncBatch<NoteResource>,
    pub files: SyncBatch<FileResource>,
    pub folders: SyncBatch<FolderResource>,
    pub settings: SyncBatch<SettingResource>,
}

impl SyncBatches {
    pub fn from_snapshot(
        snapshot: VaultSnapshot,
        store: &LocalStore,
        scan_options: &VaultScanOptions,
        context: Option<String>,
        missing_path_mode: MissingPathMode,
    ) -> Result<Self> {
        let note_time = store.sync_time(ResourceKind::Note)?;
        let file_time = store.sync_time(ResourceKind::File)?;
        let folder_time = store.sync_time(ResourceKind::Folder)?;
        let setting_time = store.sync_time(ResourceKind::Setting)?;

        let note_missing = missing_or_deleted(
            ResourceKind::Note,
            note_paths(&snapshot.notes),
            store,
            scan_options,
        )?;
        let file_missing = missing_or_deleted(
            ResourceKind::File,
            file_paths(&snapshot.files),
            store,
            scan_options,
        )?;
        let folder_missing = missing_or_deleted(
            ResourceKind::Folder,
            folder_paths(&snapshot.folders),
            store,
            scan_options,
        )?;
        let setting_missing = missing_or_deleted(
            ResourceKind::Setting,
            text_paths(&snapshot.settings),
            store,
            scan_options,
        )?;

        Ok(Self {
            notes: apply_missing_mode(
                text_batch(
                    filter_text_resources(snapshot.notes, ResourceKind::Note, note_time, store)?,
                    note_time,
                    context.clone(),
                ),
                note_missing,
                missing_path_mode,
            ),
            files: apply_missing_mode(
                file_batch(
                    filter_file_resources(snapshot.files, file_time, store)?,
                    file_time,
                    context.clone(),
                ),
                file_missing,
                missing_path_mode,
            ),
            folders: apply_missing_mode(
                folder_batch(
                    filter_folder_resources(snapshot.folders, folder_time, store)?,
                    folder_time,
                    context.clone(),
                ),
                folder_missing,
                missing_path_mode,
            ),
            settings: apply_missing_mode(
                text_batch(
                    filter_text_resources(
                        snapshot.settings,
                        ResourceKind::Setting,
                        setting_time,
                        store,
                    )?,
                    setting_time,
                    context,
                ),
                setting_missing,
                missing_path_mode,
            ),
        })
    }
}

fn filter_text_resources<T>(
    items: Vec<T>,
    kind: ResourceKind,
    last_time: RemoteMillis,
    store: &LocalStore,
) -> Result<Vec<T>>
where
    T: TextResourceView,
{
    items
        .into_iter()
        .filter_map(|item| {
            match text_is_unchanged(kind, item.path(), item.content_hash(), last_time, store) {
                Ok(true) => None,
                Ok(false) => Some(Ok(item)),
                Err(err) => Some(Err(err)),
            }
        })
        .collect()
}

fn filter_file_resources(
    items: Vec<FileResource>,
    last_time: RemoteMillis,
    store: &LocalStore,
) -> Result<Vec<FileResource>> {
    items
        .into_iter()
        .filter_map(|item| {
            match file_is_unchanged(
                ResourceKind::File,
                &item.path,
                &item.content_hash,
                item.size,
                last_time,
                store,
            ) {
                Ok(true) => None,
                Ok(false) => Some(Ok(item)),
                Err(err) => Some(Err(err)),
            }
        })
        .collect()
}

fn filter_folder_resources(
    items: Vec<FolderResource>,
    last_time: RemoteMillis,
    store: &LocalStore,
) -> Result<Vec<FolderResource>> {
    if last_time.as_i64() == 0 {
        return Ok(items);
    }

    items
        .into_iter()
        .filter_map(
            |item| match store.hash_entry(ResourceKind::Folder, &item.path) {
                Ok(None) => Some(Ok(item)),
                Ok(Some(_)) => None,
                Err(err) => Some(Err(err.into())),
            },
        )
        .collect()
}

fn text_is_unchanged(
    kind: ResourceKind,
    path: &VaultPath,
    content_hash: &ContentHash,
    last_time: RemoteMillis,
    store: &LocalStore,
) -> Result<bool> {
    if store.has_pending_modify(kind, path)? {
        return Ok(false);
    }

    if last_time.as_i64() == 0 {
        return Ok(false);
    }

    let Some(entry) = store.hash_entry(kind, path)? else {
        return Ok(false);
    };

    Ok(entry.content_hash()?.as_ref() == Some(content_hash))
}

fn file_is_unchanged(
    kind: ResourceKind,
    path: &VaultPath,
    content_hash: &ContentHash,
    size: u64,
    last_time: RemoteMillis,
    store: &LocalStore,
) -> Result<bool> {
    if store.has_pending_modify(kind, path)? {
        return Ok(false);
    }

    if last_time.as_i64() == 0 {
        return Ok(false);
    }

    let Some(entry) = store.hash_entry(kind, path)? else {
        return Ok(false);
    };

    Ok(entry.size == size && entry.content_hash()?.as_ref() == Some(content_hash))
}

fn missing_or_deleted(
    kind: ResourceKind,
    current_paths: BTreeSet<VaultPath>,
    store: &LocalStore,
    scan_options: &VaultScanOptions,
) -> Result<Vec<DeletedResource>> {
    store
        .all_hash_paths(kind)?
        .into_iter()
        .filter(|path| is_managed_path(kind, path, scan_options))
        .filter(|path| !current_paths.contains(path))
        .map(|path| {
            Ok(DeletedResource {
                path_hash: path_hash(path.as_str())?,
                path,
            })
        })
        .collect()
}

fn is_managed_path(kind: ResourceKind, path: &VaultPath, scan_options: &VaultScanOptions) -> bool {
    if scan_options.should_ignore(path) {
        return false;
    }

    match kind {
        ResourceKind::Setting => scan_options.is_setting_path(path),
        ResourceKind::Note => is_note_path(path) && !scan_options.is_setting_path(path),
        ResourceKind::File => !is_note_path(path) && !scan_options.is_setting_path(path),
        ResourceKind::Folder => !scan_options.is_setting_path(path),
    }
}

fn is_note_path(path: &VaultPath) -> bool {
    path.as_str()
        .rsplit_once('.')
        .is_some_and(|(_, extension)| extension.eq_ignore_ascii_case("md"))
}

fn apply_missing_mode<T>(
    mut batch: SyncBatch<T>,
    paths: Vec<DeletedResource>,
    mode: MissingPathMode,
) -> SyncBatch<T> {
    match mode {
        MissingPathMode::Missing => batch.missing = paths,
        MissingPathMode::Deleted => batch.deleted = paths,
    }

    batch
}

fn note_paths(items: &[NoteResource]) -> BTreeSet<VaultPath> {
    text_paths(items)
}

fn text_paths<T>(items: &[T]) -> BTreeSet<VaultPath>
where
    T: TextResourceView,
{
    items.iter().map(|item| item.path().clone()).collect()
}

fn file_paths(items: &[FileResource]) -> BTreeSet<VaultPath> {
    items.iter().map(|item| item.path.clone()).collect()
}

fn folder_paths(items: &[FolderResource]) -> BTreeSet<VaultPath> {
    items.iter().map(|item| item.path.clone()).collect()
}

trait TextResourceView {
    fn path(&self) -> &VaultPath;
    fn content_hash(&self) -> &ContentHash;
}

impl TextResourceView for TextResource {
    fn path(&self) -> &VaultPath {
        &self.path
    }

    fn content_hash(&self) -> &ContentHash {
        &self.content_hash
    }
}

fn text_batch<T>(items: Vec<T>, last_time: RemoteMillis, context: Option<String>) -> SyncBatch<T> {
    let mut batch = SyncBatch::new(last_time);
    batch.context = context;
    batch.items = items;
    batch
}

fn file_batch(
    items: Vec<FileResource>,
    last_time: RemoteMillis,
    context: Option<String>,
) -> SyncBatch<FileResource> {
    let mut batch = SyncBatch::new(last_time);
    batch.context = context;
    batch.items = items;
    batch
}

fn folder_batch(
    items: Vec<FolderResource>,
    last_time: RemoteMillis,
    context: Option<String>,
) -> SyncBatch<FolderResource> {
    let mut batch = SyncBatch::new(last_time);
    batch.context = context;
    batch.items = items;
    batch
}
