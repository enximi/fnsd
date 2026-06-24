use crate::config::AppConfig;
use crate::core::{VaultName, VaultPath};
use crate::store::LocalStore;
use crate::vault::fs::VaultFs;
use crate::vault::watch::VaultWatchEvent;
use crate::ws::WebSocketClient;
use tracing::warn;

use crate::sync::session::Result;

mod change;
mod delete;
mod rename;

pub(crate) async fn send_local_changes(
    ws: &mut WebSocketClient,
    vault_name: &VaultName,
    vault: &VaultFs,
    store: &mut LocalStore,
    config: &AppConfig,
    events: Vec<VaultWatchEvent>,
) -> Result<()> {
    let mut sender = LocalChangeSender {
        ws,
        vault_name,
        vault,
        store,
        config,
    };

    for event in events {
        sender.send_event(event).await?;
    }

    Ok(())
}

pub(crate) async fn send_local_change(
    ws: &mut WebSocketClient,
    vault_name: &VaultName,
    vault: &VaultFs,
    store: &mut LocalStore,
    config: &AppConfig,
    event: VaultWatchEvent,
) -> Result<()> {
    LocalChangeSender {
        ws,
        vault_name,
        vault,
        store,
        config,
    }
    .send_event(event)
    .await
}

pub(super) struct LocalChangeSender<'a> {
    ws: &'a mut WebSocketClient,
    vault_name: &'a VaultName,
    vault: &'a VaultFs,
    store: &'a mut LocalStore,
    config: &'a AppConfig,
}

impl LocalChangeSender<'_> {
    async fn send_event(&mut self, event: VaultWatchEvent) -> Result<()> {
        match event {
            VaultWatchEvent::Changed { path } => {
                self.send_path_change(&path).await?;
            }
            VaultWatchEvent::RenameFrom { path } | VaultWatchEvent::RenameTo { path } => {
                self.send_path_change(&path).await?;
            }
            VaultWatchEvent::Renamed { old_path, new_path } => {
                self.send_path_rename(&old_path, &new_path).await?;
            }
            VaultWatchEvent::RescanNeeded => {
                warn!("rescan-needed watch event ignored by long-lived session");
            }
        }

        Ok(())
    }
}

pub(super) fn is_note_path(path: &VaultPath) -> bool {
    path.as_str()
        .rsplit_once('.')
        .is_some_and(|(_, extension)| extension.eq_ignore_ascii_case("md"))
}
