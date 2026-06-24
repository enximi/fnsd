use fns_config::AppConfig;
use fns_core::{RemoteMillis, ResourceKind, VaultName, VaultPath};
use fns_hash::{file_content_hash, path_hash, text_content_hash};
use fns_local_store::LocalStore;
use fns_protocol::{
    Action, FileDeleteRequest, FileUploadCheckRequest, FolderCreateRequest, FolderDeleteRequest,
    NoteDeleteRequest, SettingDeleteRequest,
};
use fns_sync_plan::{build_note_modify_request, build_setting_modify_request};
use fns_vault_fs::VaultFs;
use fns_vault_watch::VaultWatchEvent;
use fns_ws_client::FnsWsClient;
use tracing::{debug, warn};

use crate::Result;

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
        VaultWatchEvent::RescanNeeded => {
            warn!("rescan-needed watch event ignored by long-lived session");
        }
    }

    Ok(())
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
            send_folder_modify(ws, vault_name, store, path, &metadata).await?;
        }
        Some(metadata) if metadata.file_type().is_file() => {
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
            send_delete_by_known_kind(ws, vault_name, store, path).await?;
        }
    }

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
