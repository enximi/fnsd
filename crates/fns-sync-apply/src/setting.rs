use fns_core::{RemoteMillis, ResourceKind, VaultPath};
use fns_local_store::LocalStore;
use fns_protocol::{
    SettingDeleteAckMessage, SettingModifyAckMessage, SettingSyncDeleteMessage,
    SettingSyncEndMessage, SettingSyncModifyMessage, SettingSyncMtimeMessage,
    SettingSyncNeedUploadMessage, TextFrame,
};
use fns_sync_plan::{
    SettingOperation, plan_setting_delete, plan_setting_modify, plan_setting_mtime,
    plan_setting_need_upload,
};
use fns_vault_fs::VaultFs;

use crate::{EventOutcome, Result, local};

pub(crate) fn apply_modify(
    frame: &TextFrame,
    vault: &VaultFs,
    store: &mut LocalStore,
) -> Result<EventOutcome> {
    let message: SettingSyncModifyMessage = frame.decode_payload()?;
    let SettingOperation::Write(text) = plan_setting_modify(&message)? else {
        unreachable!("setting modify planner must produce write operation");
    };
    local::apply_remote_text(ResourceKind::Setting, text, vault, store)
}

pub(crate) fn apply_delete(
    frame: &TextFrame,
    vault: &VaultFs,
    store: &mut LocalStore,
) -> Result<EventOutcome> {
    let message: SettingSyncDeleteMessage = frame.decode_payload()?;
    let last_time = RemoteMillis::new(message.last_time)?;
    let SettingOperation::Delete(resource) = plan_setting_delete(&message)? else {
        unreachable!("setting delete planner must produce delete operation");
    };
    local::apply_file_delete(ResourceKind::Setting, resource, last_time, vault, store)
}

pub(crate) fn apply_mtime(
    frame: &TextFrame,
    vault: &VaultFs,
    store: &mut LocalStore,
) -> Result<EventOutcome> {
    let message: SettingSyncMtimeMessage = frame.decode_payload()?;
    let SettingOperation::UpdateMtime(update) = plan_setting_mtime(&message)? else {
        unreachable!("setting mtime planner must produce mtime operation");
    };
    local::apply_mtime_update(ResourceKind::Setting, update, vault, store)
}

pub(crate) fn need_upload(frame: &TextFrame) -> Result<EventOutcome> {
    let message: SettingSyncNeedUploadMessage = frame.decode_payload()?;
    let SettingOperation::Upload(path) = plan_setting_need_upload(&message)? else {
        unreachable!("setting need-upload planner must produce upload operation");
    };
    Ok(EventOutcome::NeedSettingUpload(path))
}

pub(crate) fn sync_end(frame: &TextFrame, store: &mut LocalStore) -> Result<EventOutcome> {
    let message: SettingSyncEndMessage = frame.decode_payload()?;
    local::sync_end(ResourceKind::Setting, message.last_time, store)
}

pub(crate) fn modify_ack(frame: &TextFrame, store: &mut LocalStore) -> Result<EventOutcome> {
    let message: SettingModifyAckMessage = frame.decode_payload()?;
    let path = VaultPath::new(&message.path)?;
    let last_time = RemoteMillis::new(message.last_time)?;
    store.remove_pending_modify(ResourceKind::Setting, &path);
    store.set_sync_time(ResourceKind::Setting, last_time);
    Ok(EventOutcome::Ack {
        kind: ResourceKind::Setting,
        path,
    })
}

pub(crate) fn delete_ack(frame: &TextFrame, store: &mut LocalStore) -> Result<EventOutcome> {
    let message: SettingDeleteAckMessage = frame.decode_payload()?;
    local::delete_ack(
        ResourceKind::Setting,
        &message.path,
        message.last_time,
        store,
    )
}
