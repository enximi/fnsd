use fns_config::AppConfig;

use fns_core::{RemoteMillis, ResourceKind, VaultName, VaultPath};
use fns_hash::{file_content_hash, path_hash, text_content_hash};
use fns_local_store::{LocalStore, PendingRename};
use fns_protocol::{
    Action, FileDeleteRequest, FileRenameRequest, FileUploadCheckRequest, FolderCreateRequest,
    FolderDeleteRequest, FolderRenameRequest, NoteDeleteRequest, NoteRenameRequest,
    SettingDeleteRequest,
};
use fns_sync_plan::{build_note_modify_request, build_setting_modify_request};
use fns_vault_fs::{VaultFs, VaultScanOptions};
use fns_vault_watch::VaultWatchEvent;
use fns_ws_client::FnsWsClient;
use tracing::{debug, warn};

use crate::Result;

pub(crate) async fn send_local_changes(
    ws: &mut FnsWsClient,
    vault_name: &VaultName,
    vault: &VaultFs,
    store: &mut LocalStore,
    config: &AppConfig,
    events: Vec<VaultWatchEvent>,
) -> Result<()> {
    for event in events {
        send_local_change(ws, vault_name, vault, store, config, event).await?;
    }

    Ok(())
}

pub(crate) async fn send_local_change(
    ws: &mut FnsWsClient,
    vault_name: &VaultName,
    vault: &VaultFs,
    store: &mut LocalStore,
    config: &AppConfig,
    event: VaultWatchEvent,
) -> Result<()> {
    match event {
        VaultWatchEvent::Changed { path } => {
            send_path_change(ws, vault_name, vault, store, config, &path).await?;
        }
        VaultWatchEvent::RenameFrom { path } | VaultWatchEvent::RenameTo { path } => {
            send_path_change(ws, vault_name, vault, store, config, &path).await?;
        }
        VaultWatchEvent::Renamed { old_path, new_path } => {
            send_path_rename(ws, vault_name, vault, store, config, &old_path, &new_path).await?;
        }
        VaultWatchEvent::RescanNeeded => {
            warn!("rescan-needed watch event ignored by long-lived session");
        }
    }

    Ok(())
}

async fn send_path_rename(
    ws: &mut FnsWsClient,
    vault_name: &VaultName,
    vault: &VaultFs,
    store: &mut LocalStore,
    config: &AppConfig,
    old_path: &VaultPath,
    new_path: &VaultPath,
) -> Result<()> {
    let scan_options = config.scan_options()?;

    let old_ignored = scan_options.should_ignore(old_path);
    let new_ignored = scan_options.should_ignore(new_path);
    match (old_ignored, new_ignored) {
        (true, true) => return Ok(()),
        (true, false) => {
            return send_path_change(ws, vault_name, vault, store, config, new_path).await;
        }
        (false, true) => {
            return send_path_change(ws, vault_name, vault, store, config, old_path).await;
        }
        (false, false) => {}
    }

    if store.hash_entry(ResourceKind::Note, old_path).is_some() {
        if can_rename_note(vault, store, old_path, new_path)? {
            return send_note_rename(ws, vault_name, store, old_path, new_path).await;
        }
    } else if store.hash_entry(ResourceKind::File, old_path).is_some() {
        if can_rename_file(vault, store, old_path, new_path)?
            && !scan_options.is_setting_path(new_path)
        {
            return send_file_rename(ws, vault_name, store, old_path, new_path).await;
        }
    } else if store.hash_entry(ResourceKind::Folder, old_path).is_some() {
        if can_rename_folder(vault, old_path, new_path) {
            return send_folder_rename(ws, vault_name, store, old_path, new_path).await;
        }
    }

    send_path_change(ws, vault_name, vault, store, config, old_path).await?;
    send_path_change(ws, vault_name, vault, store, config, new_path).await
}

async fn send_path_change(
    ws: &mut FnsWsClient,
    vault_name: &VaultName,
    vault: &VaultFs,
    store: &mut LocalStore,
    config: &AppConfig,
    path: &VaultPath,
) -> Result<()> {
    let scan_options = config.scan_options()?;

    if scan_options.should_ignore(path) {
        return Ok(());
    }

    let absolute = path.to_path_buf_under(vault.root());
    let metadata = std::fs::symlink_metadata(&absolute).ok();

    match metadata {
        Some(metadata) if metadata.file_type().is_dir() => {
            if is_current_path_unchanged(vault, store, &scan_options, path, &metadata)? {
                return Ok(());
            }
            send_folder_modify(ws, vault_name, store, path, &metadata).await?;
        }
        Some(metadata) if metadata.file_type().is_file() => {
            if is_current_path_unchanged(vault, store, &scan_options, path, &metadata)? {
                return Ok(());
            }
            if scan_options.is_setting_path(path) {
                send_setting_modify(ws, vault_name, vault, store, path).await?;
            } else if is_note_path(path) {
                send_note_modify(ws, vault_name, vault, store, path).await?;
            } else {
                send_file_upload_check(ws, vault_name, vault, store, path).await?;
            }
        }
        Some(_) => {}
        None => {
            if is_deleted_path_unchanged(store, path) {
                return Ok(());
            }
            send_delete_by_known_kind(ws, vault_name, store, path).await?;
        }
    }

    Ok(())
}

async fn send_note_rename(
    ws: &mut FnsWsClient,
    vault_name: &VaultName,
    store: &mut LocalStore,
    old_path: &VaultPath,
    new_path: &VaultPath,
) -> Result<()> {
    let content_hash = store
        .hash_entry(ResourceKind::Note, old_path)
        .and_then(|entry| entry.content_hash().ok().flatten());
    let request = NoteRenameRequest {
        vault: vault_name.to_string(),
        path: new_path.to_string(),
        path_hash: path_hash(new_path.as_str())?.to_string(),
        old_path: old_path.to_string(),
        old_path_hash: path_hash(old_path.as_str())?.to_string(),
    };

    ws.send_json(Action::NoteRename, &request).await?;
    move_hash_entry(ResourceKind::Note, old_path, new_path, content_hash, store);
    store.save()?;
    debug!(old_path = %old_path, new_path = %new_path, "sent note rename from watch event");
    Ok(())
}

async fn send_file_rename(
    ws: &mut FnsWsClient,
    vault_name: &VaultName,
    store: &mut LocalStore,
    old_path: &VaultPath,
    new_path: &VaultPath,
) -> Result<()> {
    let content_hash = store
        .hash_entry(ResourceKind::File, old_path)
        .and_then(|entry| entry.content_hash().ok().flatten());
    let request = FileRenameRequest {
        vault: vault_name.to_string(),
        path: new_path.to_string(),
        path_hash: path_hash(new_path.as_str())?.to_string(),
        old_path: old_path.to_string(),
        old_path_hash: path_hash(old_path.as_str())?.to_string(),
    };

    ws.send_json(Action::FileRename, &request).await?;
    move_hash_entry(ResourceKind::File, old_path, new_path, content_hash, store);
    store.save()?;
    debug!(old_path = %old_path, new_path = %new_path, "sent file rename from watch event");
    Ok(())
}

async fn send_folder_rename(
    ws: &mut FnsWsClient,
    vault_name: &VaultName,
    store: &mut LocalStore,
    old_path: &VaultPath,
    new_path: &VaultPath,
) -> Result<()> {
    let request = FolderRenameRequest {
        vault: vault_name.to_string(),
        path: new_path.to_string(),
        path_hash: path_hash(new_path.as_str())?.to_string(),
        old_path: old_path.to_string(),
        old_path_hash: path_hash(old_path.as_str())?.to_string(),
    };

    ws.send_json(Action::FolderRename, &request).await?;
    store.rename_hash_tree(old_path, new_path);
    store.push_pending_rename(
        ResourceKind::Folder,
        PendingRename::new(old_path.clone(), new_path.clone(), None),
    );
    store.save()?;
    debug!(old_path = %old_path, new_path = %new_path, "sent folder rename from watch event");
    Ok(())
}

async fn send_note_modify(
    ws: &mut FnsWsClient,
    vault_name: &VaultName,
    vault: &VaultFs,
    store: &mut LocalStore,
    path: &VaultPath,
) -> Result<()> {
    let content = vault.read_text(path)?;
    let metadata = vault.file_metadata(path)?;
    let content_hash = text_content_hash(&content);
    let request = build_note_modify_request(
        vault_name,
        path,
        content,
        &content_hash,
        metadata.ctime,
        metadata.mtime,
    )?;

    ws.send_json(Action::NoteModify, &request).await?;
    store.set_pending_modify(ResourceKind::Note, path, &content_hash);
    store.set_content_hash(
        ResourceKind::Note,
        path,
        Some(content_hash),
        metadata.mtime,
        metadata.size,
    );
    store.save()?;
    debug!(path = %path, "sent note modify from watch event");
    Ok(())
}

async fn send_setting_modify(
    ws: &mut FnsWsClient,
    vault_name: &VaultName,
    vault: &VaultFs,
    store: &mut LocalStore,
    path: &VaultPath,
) -> Result<()> {
    let content = vault.read_text(path)?;
    let metadata = vault.file_metadata(path)?;
    let content_hash = text_content_hash(&content);
    let request = build_setting_modify_request(
        vault_name,
        path,
        content,
        &content_hash,
        metadata.ctime,
        metadata.mtime,
    )?;

    ws.send_json(Action::SettingModify, &request).await?;
    store.set_pending_modify(ResourceKind::Setting, path, &content_hash);
    store.set_content_hash(
        ResourceKind::Setting,
        path,
        Some(content_hash),
        metadata.mtime,
        metadata.size,
    );
    store.save()?;
    debug!(path = %path, "sent setting modify from watch event");
    Ok(())
}

async fn send_file_upload_check(
    ws: &mut FnsWsClient,
    vault_name: &VaultName,
    vault: &VaultFs,
    store: &mut LocalStore,
    path: &VaultPath,
) -> Result<()> {
    let bytes = vault.read_bytes(path)?;
    let metadata = vault.file_metadata(path)?;
    let content_hash = file_content_hash(&bytes);
    let request = FileUploadCheckRequest {
        vault: vault_name.to_string(),
        path: path.to_string(),
        path_hash: path_hash(path.as_str())?.to_string(),
        content_hash: content_hash.to_string(),
        size: i64::try_from(metadata.size).unwrap_or(i64::MAX),
        ctime: metadata.ctime.as_i64(),
        mtime: metadata.mtime.as_i64(),
    };

    ws.send_json(Action::FileUploadCheck, &request).await?;
    store.set_content_hash(
        ResourceKind::File,
        path,
        Some(content_hash),
        metadata.mtime,
        metadata.size,
    );
    store.save()?;
    debug!(path = %path, "sent file upload check from watch event");
    Ok(())
}

async fn send_folder_modify(
    ws: &mut FnsWsClient,
    vault_name: &VaultName,
    store: &mut LocalStore,
    path: &VaultPath,
    metadata: &std::fs::Metadata,
) -> Result<()> {
    let request = FolderCreateRequest {
        vault: vault_name.to_string(),
        path: path.to_string(),
        path_hash: path_hash(path.as_str())?.to_string(),
    };

    ws.send_json(Action::FolderModify, &request).await?;
    store.set_content_hash(
        ResourceKind::Folder,
        path,
        None,
        modified_millis(metadata),
        0,
    );
    store.save()?;
    debug!(path = %path, "sent folder modify from watch event");
    Ok(())
}

async fn send_delete_by_known_kind(
    ws: &mut FnsWsClient,
    vault_name: &VaultName,
    store: &mut LocalStore,
    path: &VaultPath,
) -> Result<()> {
    if store.hash_entry(ResourceKind::Note, path).is_some() {
        let request = NoteDeleteRequest {
            vault: vault_name.to_string(),
            path: path.to_string(),
            path_hash: path_hash(path.as_str())?.to_string(),
        };
        ws.send_json(Action::NoteDelete, &request).await?;
        store.insert_pending_delete(ResourceKind::Note, path);
    } else if store.hash_entry(ResourceKind::File, path).is_some() {
        let request = FileDeleteRequest {
            vault: vault_name.to_string(),
            path: path.to_string(),
            path_hash: path_hash(path.as_str())?.to_string(),
        };
        ws.send_json(Action::FileDelete, &request).await?;
        store.insert_pending_delete(ResourceKind::File, path);
    } else if store.hash_entry(ResourceKind::Setting, path).is_some() {
        let request = SettingDeleteRequest {
            vault: vault_name.to_string(),
            path: path.to_string(),
            path_hash: path_hash(path.as_str())?.to_string(),
        };
        ws.send_json(Action::SettingDelete, &request).await?;
        store.insert_pending_delete(ResourceKind::Setting, path);
    } else if store.hash_entry(ResourceKind::Folder, path).is_some() {
        let request = FolderDeleteRequest {
            vault: vault_name.to_string(),
            path: path.to_string(),
            path_hash: path_hash(path.as_str())?.to_string(),
        };
        ws.send_json(Action::FolderDelete, &request).await?;
        store.insert_pending_delete(ResourceKind::Folder, path);
    }

    store.save()?;
    debug!(path = %path, "sent delete from watch event");
    Ok(())
}

fn move_hash_entry(
    kind: ResourceKind,
    old_path: &VaultPath,
    new_path: &VaultPath,
    content_hash: Option<fns_core::ContentHash>,
    store: &mut LocalStore,
) {
    let entry = store.remove_hash_entry(kind, old_path);
    if let Some(entry) = entry {
        store.set_hash_entry(kind, new_path, entry);
    }
    store.push_pending_rename(
        kind,
        PendingRename::new(old_path.clone(), new_path.clone(), content_hash),
    );
}

fn can_rename_note(
    vault: &VaultFs,
    store: &LocalStore,
    old_path: &VaultPath,
    new_path: &VaultPath,
) -> Result<bool> {
    if !new_path.to_path_buf_under(vault.root()).is_file() || !is_note_path(new_path) {
        return Ok(false);
    }

    let Some(old_hash) = store
        .hash_entry(ResourceKind::Note, old_path)
        .and_then(|entry| entry.content_hash().ok().flatten())
    else {
        return Ok(true);
    };

    let content = vault.read_text(new_path)?;
    Ok(text_content_hash(&content) == old_hash)
}

fn can_rename_file(
    vault: &VaultFs,
    store: &LocalStore,
    old_path: &VaultPath,
    new_path: &VaultPath,
) -> Result<bool> {
    if !new_path.to_path_buf_under(vault.root()).is_file() || is_note_path(new_path) {
        return Ok(false);
    }

    let Some(old_hash) = store
        .hash_entry(ResourceKind::File, old_path)
        .and_then(|entry| entry.content_hash().ok().flatten())
    else {
        return Ok(true);
    };

    let bytes = vault.read_bytes(new_path)?;
    Ok(file_content_hash(&bytes) == old_hash)
}

fn can_rename_folder(vault: &VaultFs, old_path: &VaultPath, new_path: &VaultPath) -> bool {
    let new_absolute = new_path.to_path_buf_under(vault.root());
    new_absolute.is_dir() && !old_path.as_str().is_empty()
}

fn is_current_path_unchanged(
    vault: &VaultFs,
    store: &LocalStore,
    scan_options: &VaultScanOptions,
    path: &VaultPath,
    metadata: &std::fs::Metadata,
) -> Result<bool> {
    if metadata.file_type().is_dir() {
        return Ok(store.hash_entry(ResourceKind::Folder, path).is_some());
    }

    if scan_options.is_setting_path(path) {
        return text_file_unchanged(ResourceKind::Setting, vault, store, path);
    }

    if is_note_path(path) {
        return text_file_unchanged(ResourceKind::Note, vault, store, path);
    }

    binary_file_unchanged(vault, store, path)
}

fn text_file_unchanged(
    kind: ResourceKind,
    vault: &VaultFs,
    store: &LocalStore,
    path: &VaultPath,
) -> Result<bool> {
    let Some(entry) = store.hash_entry(kind, path) else {
        return Ok(false);
    };
    let content = vault.read_text(path)?;
    Ok(entry.content_hash()?.as_ref() == Some(&text_content_hash(&content)))
}

fn binary_file_unchanged(vault: &VaultFs, store: &LocalStore, path: &VaultPath) -> Result<bool> {
    let Some(entry) = store.hash_entry(ResourceKind::File, path) else {
        return Ok(false);
    };
    let bytes = vault.read_bytes(path)?;
    Ok(
        entry.content_hash()?.as_ref() == Some(&file_content_hash(&bytes))
            && entry.size == bytes.len() as u64,
    )
}

fn is_deleted_path_unchanged(store: &LocalStore, path: &VaultPath) -> bool {
    [
        ResourceKind::Note,
        ResourceKind::File,
        ResourceKind::Folder,
        ResourceKind::Setting,
    ]
    .into_iter()
    .all(|kind| store.hash_entry(kind, path).is_none())
}

fn is_note_path(path: &VaultPath) -> bool {
    path.as_str()
        .rsplit_once('.')
        .is_some_and(|(_, extension)| extension.eq_ignore_ascii_case("md"))
}

fn modified_millis(metadata: &std::fs::Metadata) -> RemoteMillis {
    metadata
        .modified()
        .ok()
        .and_then(system_time_millis)
        .unwrap_or_else(|| RemoteMillis::new(0).expect("zero timestamp is valid"))
}

fn system_time_millis(time: std::time::SystemTime) -> Option<RemoteMillis> {
    let duration = time.duration_since(std::time::UNIX_EPOCH).ok()?;
    let millis = i64::try_from(duration.as_millis()).ok()?;
    RemoteMillis::new(millis).ok()
}
