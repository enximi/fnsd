use std::path::PathBuf;
use std::thread;

use fns_vault_fs::VaultScanOptions;
use fns_vault_watch::{VaultWatchEvent, VaultWatcher};
use tokio::sync::mpsc;
use tracing::{debug, warn};

use crate::Result;

pub(crate) type WatchReceiver = mpsc::Receiver<Result<VaultWatchEvent>>;

pub(crate) fn spawn_watch_task(root: PathBuf, options: VaultScanOptions) -> WatchReceiver {
    let (sender, receiver) = mpsc::channel(1024);

    thread::spawn(move || {
        let watcher = match VaultWatcher::new(&root, options) {
            Ok(watcher) => watcher,
            Err(err) => {
                let _ = sender.blocking_send(Err(err.into()));
                return;
            }
        };

        debug!(root = %root.display(), "vault watcher started");

        loop {
            match watcher.recv() {
                Ok(event) => {
                    if sender.blocking_send(Ok(event)).is_err() {
                        break;
                    }
                }
                Err(err) => {
                    warn!(%err, "vault watcher stopped");
                    let _ = sender.blocking_send(Err(err.into()));
                    break;
                }
            }
        }
    });

    receiver
}
