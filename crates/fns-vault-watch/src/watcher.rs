use std::path::{Component, Path, PathBuf};
use std::sync::mpsc::{Receiver, TryRecvError, channel};

use fns_core::VaultPath;
use fns_vault_fs::VaultScanOptions;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tracing::debug;

use crate::{Result, VaultWatchError, error::io, event::VaultWatchEvent};

pub struct VaultWatcher {
    receiver: Receiver<Result<VaultWatchEvent>>,
    _watcher: RecommendedWatcher,
}

impl VaultWatcher {
    pub fn new(root: impl AsRef<Path>, options: VaultScanOptions) -> Result<Self> {
        let root = canonical_root(root.as_ref())?;
        let (sender, receiver) = channel();
        let callback_root = root.clone();
        let callback_options = options.clone();

        let mut watcher =
            notify::recommended_watcher(move |event: notify::Result<Event>| match event {
                Ok(event) => {
                    for event in normalize_event(&callback_root, &callback_options, event) {
                        let _ = sender.send(Ok(event));
                    }
                }
                Err(err) => {
                    let _ = sender.send(Err(VaultWatchError::Notify(err)));
                }
            })?;

        watcher.watch(&root, RecursiveMode::Recursive)?;

        Ok(Self {
            receiver,
            _watcher: watcher,
        })
    }

    pub fn recv(&self) -> Result<VaultWatchEvent> {
        self.receiver.recv()?
    }

    pub fn try_recv(&self) -> Result<Option<VaultWatchEvent>> {
        match self.receiver.try_recv() {
            Ok(event) => event.map(Some),
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => Err(VaultWatchError::ChannelClosed),
        }
    }
}

fn canonical_root(root: &Path) -> Result<PathBuf> {
    let metadata = std::fs::metadata(root).map_err(|err| io(root, err))?;
    if !metadata.is_dir() {
        return Err(io(
            root,
            std::io::Error::new(
                std::io::ErrorKind::NotADirectory,
                "vault root is not a directory",
            ),
        ));
    }

    root.canonicalize().map_err(|err| io(root, err))
}

fn normalize_event(root: &Path, options: &VaultScanOptions, event: Event) -> Vec<VaultWatchEvent> {
    if matches!(event.kind, EventKind::Access(_)) {
        return Vec::new();
    }

    if event.paths.is_empty() {
        return vec![VaultWatchEvent::RescanNeeded];
    }

    let mut events = Vec::new();

    for path in event.paths {
        match vault_path_from_event_path(root, &path) {
            PathNormalization::Vault(path) => {
                if options.should_ignore(&path) {
                    debug!(path = %path, "ignored vault watch event");
                    continue;
                }

                events.push(VaultWatchEvent::Changed { path });
            }
            PathNormalization::Root => events.push(VaultWatchEvent::RescanNeeded),
            PathNormalization::OutsideRoot => {}
            PathNormalization::Unknown => events.push(VaultWatchEvent::RescanNeeded),
        }
    }

    events
}

enum PathNormalization {
    Vault(VaultPath),
    Root,
    OutsideRoot,
    Unknown,
}

fn vault_path_from_event_path(root: &Path, path: &Path) -> PathNormalization {
    let relative = if path.is_absolute() {
        match path.strip_prefix(root) {
            Ok(relative) => relative,
            Err(_) => return PathNormalization::OutsideRoot,
        }
    } else {
        path
    };

    let mut parts = Vec::new();

    for component in relative.components() {
        match component {
            Component::Normal(value) => {
                let Some(value) = value.to_str() else {
                    return PathNormalization::Unknown;
                };
                parts.push(value);
            }
            Component::CurDir => {}
            Component::Prefix(_) | Component::RootDir | Component::ParentDir => {
                return PathNormalization::Unknown;
            }
        }
    }

    if parts.is_empty() {
        return PathNormalization::Root;
    }

    match VaultPath::new(parts.join("/")) {
        Ok(path) => PathNormalization::Vault(path),
        Err(_) => PathNormalization::Unknown,
    }
}
