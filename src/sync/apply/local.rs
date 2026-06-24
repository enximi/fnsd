use std::io::ErrorKind;

use crate::core::{RemoteMillis, ResourceKind, VaultPath};
use crate::store::LocalStore;
use crate::sync::plan::{DeletedResource, MtimeUpdate, RemoteText, RemoteTextRename};
use crate::vault::fs::{VaultFileTimes, VaultFs, VaultFsError};

use crate::sync::apply::{EventOutcome, Result};

pub(crate) fn apply_remote_text(
    kind: ResourceKind,
    text: RemoteText,
    vault: &VaultFs,
    store: &mut LocalStore,
) -> Result<EventOutcome> {
    vault.write_text(
        &text.path,
        &text.content,
        Some(VaultFileTimes::new(Some(text.ctime), Some(text.mtime))),
    )?;
    store.set_content_hash(
        kind,
        &text.path,
        Some(text.content_hash),
        text.mtime,
        text.content.len() as u64,
    );
    store.set_sync_time(kind, text.last_time);
    Ok(EventOutcome::RemoteWrite {
        kind,
        path: text.path,
    })
}

pub(crate) fn apply_file_delete(
    kind: ResourceKind,
    resource: DeletedResource,
    last_time: RemoteMillis,
    vault: &VaultFs,
    store: &mut LocalStore,
) -> Result<EventOutcome> {
    delete_file_if_exists(vault, &resource.path)?;
    store.remove_hash_entry(kind, &resource.path);
    store.set_sync_time(kind, last_time);
    Ok(EventOutcome::RemoteDelete {
        kind,
        path: resource.path,
    })
}

pub(crate) fn apply_text_rename(
    kind: ResourceKind,
    rename: RemoteTextRename,
    vault: &VaultFs,
    store: &mut LocalStore,
) -> Result<EventOutcome> {
    rename_path(kind, &rename.old_path, &rename.path, vault, store)?;
    store.set_content_hash(
        kind,
        &rename.path,
        Some(rename.content_hash),
        rename.mtime,
        0,
    );
    store.set_sync_time(kind, rename.last_time);
    Ok(EventOutcome::RemoteRename {
        kind,
        old_path: rename.old_path,
        new_path: rename.path,
    })
}

pub(crate) fn apply_mtime_update(
    kind: ResourceKind,
    update: MtimeUpdate,
    vault: &VaultFs,
    store: &mut LocalStore,
) -> Result<EventOutcome> {
    vault.set_mtime(&update.path, update.mtime)?;

    if let Some(entry) = store.hash_entry(kind, &update.path).cloned() {
        store.set_content_hash(
            kind,
            &update.path,
            entry.content_hash()?,
            update.mtime,
            entry.size,
        );
    }

    store.set_sync_time(kind, update.last_time);
    Ok(EventOutcome::RemoteMtimeUpdate {
        kind,
        path: update.path,
    })
}

pub(crate) fn rename_path(
    kind: ResourceKind,
    old_path: &VaultPath,
    new_path: &VaultPath,
    vault: &VaultFs,
    store: &mut LocalStore,
) -> Result<()> {
    vault.rename(old_path, new_path)?;

    if let Some(entry) = store.remove_hash_entry(kind, old_path) {
        store.set_hash_entry(kind, new_path, entry);
    }

    Ok(())
}

pub(crate) fn sync_end(
    kind: ResourceKind,
    last_time: i64,
    pending_events: usize,
    store: &mut LocalStore,
) -> Result<EventOutcome> {
    let last_time = RemoteMillis::new(last_time)?;
    store.set_sync_time(kind, last_time);
    Ok(EventOutcome::SyncEnd {
        kind,
        last_time,
        pending_events,
    })
}

pub(crate) fn ack(
    kind: ResourceKind,
    path: &str,
    last_time: i64,
    store: &mut LocalStore,
) -> Result<EventOutcome> {
    let path = VaultPath::new(path)?;
    let last_time = RemoteMillis::new(last_time)?;
    store.set_sync_time(kind, last_time);
    Ok(EventOutcome::Ack { kind, path })
}

pub(crate) fn delete_ack(
    kind: ResourceKind,
    path: &str,
    last_time: i64,
    store: &mut LocalStore,
) -> Result<EventOutcome> {
    let path = VaultPath::new(path)?;
    let last_time = RemoteMillis::new(last_time)?;
    store.remove_pending_delete(kind, &path);
    store.remove_hash_entry(kind, &path);
    store.set_sync_time(kind, last_time);
    Ok(EventOutcome::Ack { kind, path })
}

pub(crate) fn commit_pending_rename(
    kind: ResourceKind,
    ack_path: &VaultPath,
    store: &mut LocalStore,
) -> Result<()> {
    let Some(rename) = store.pop_pending_rename(kind) else {
        return Ok(());
    };

    let old_path = rename.old_path()?;
    let new_path = rename.new_path()?;
    let content_hash = rename.content_hash()?;

    if &new_path != ack_path {
        return Ok(());
    }

    if let Some(entry) = store.remove_hash_entry(kind, &old_path) {
        store.set_hash_entry(kind, &new_path, entry);
    } else if let Some(content_hash) = content_hash {
        store.set_content_hash(
            kind,
            &new_path,
            Some(content_hash),
            RemoteMillis::new(0)?,
            0,
        );
    }

    Ok(())
}

pub(crate) fn delete_file_if_exists(vault: &VaultFs, path: &VaultPath) -> Result<()> {
    match vault.delete_file(path) {
        Ok(()) => Ok(()),
        Err(VaultFsError::Io { source, .. }) if source.kind() == ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err.into()),
    }
}

pub(crate) fn delete_dir_if_exists(vault: &VaultFs, path: &VaultPath) -> Result<()> {
    match vault.delete_dir_all(path) {
        Ok(()) => Ok(()),
        Err(VaultFsError::Io { source, .. }) if source.kind() == ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err.into()),
    }
}
