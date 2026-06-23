use std::path::{Path, PathBuf};

use filetime::FileTime;
use fns_core::{RemoteMillis, VaultPath};

use crate::{Result, VaultFsError, error::io};

#[derive(Debug, Clone)]
pub struct VaultFs {
    root: PathBuf,
}

impl VaultFs {
    pub fn new(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref();
        let metadata = std::fs::metadata(root).map_err(|err| io(root, err))?;

        if !metadata.is_dir() {
            return Err(VaultFsError::RootNotDirectory(root.to_path_buf()));
        }

        let root = root.canonicalize().map_err(|err| io(root, err))?;

        Ok(Self { root })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn read_text(&self, path: &VaultPath) -> Result<String> {
        let absolute = self.resolve_existing(path)?;
        std::fs::read_to_string(&absolute).map_err(|err| io(absolute, err))
    }

    pub fn read_bytes(&self, path: &VaultPath) -> Result<Vec<u8>> {
        let absolute = self.resolve_existing(path)?;
        std::fs::read(&absolute).map_err(|err| io(absolute, err))
    }

    pub fn write_text(
        &self,
        path: &VaultPath,
        content: &str,
        times: Option<VaultFileTimes>,
    ) -> Result<()> {
        let absolute = self.resolve_for_write(path)?;
        self.ensure_write_target_inside(path, &absolute)?;
        ensure_parent_dir(&absolute)?;
        std::fs::write(&absolute, content).map_err(|err| io(&absolute, err))?;
        self.apply_times(path, times)
    }

    pub fn write_bytes(
        &self,
        path: &VaultPath,
        content: &[u8],
        times: Option<VaultFileTimes>,
    ) -> Result<()> {
        let absolute = self.resolve_for_write(path)?;
        self.ensure_write_target_inside(path, &absolute)?;
        ensure_parent_dir(&absolute)?;
        std::fs::write(&absolute, content).map_err(|err| io(&absolute, err))?;
        self.apply_times(path, times)
    }

    pub fn create_dir_all(&self, path: &VaultPath) -> Result<()> {
        let absolute = self.resolve_for_write(path)?;
        self.ensure_write_target_inside(path, &absolute)?;
        std::fs::create_dir_all(&absolute).map_err(|err| io(absolute, err))
    }

    pub fn delete_file(&self, path: &VaultPath) -> Result<()> {
        let absolute = self.resolve_existing(path)?;
        std::fs::remove_file(&absolute).map_err(|err| io(absolute, err))
    }

    pub fn delete_dir_all(&self, path: &VaultPath) -> Result<()> {
        let absolute = self.resolve_existing(path)?;
        std::fs::remove_dir_all(&absolute).map_err(|err| io(absolute, err))
    }

    pub fn rename(&self, old_path: &VaultPath, new_path: &VaultPath) -> Result<()> {
        let old_absolute = self.resolve_existing(old_path)?;
        let new_absolute = self.resolve_for_write(new_path)?;
        self.ensure_write_target_inside(new_path, &new_absolute)?;
        ensure_parent_dir(&new_absolute)?;
        std::fs::rename(&old_absolute, &new_absolute).map_err(|err| io(old_absolute, err))
    }

    pub fn set_mtime(&self, path: &VaultPath, mtime: RemoteMillis) -> Result<()> {
        let absolute = self.resolve_existing(path)?;
        set_modified_time(&absolute, mtime)
    }

    pub(crate) fn resolve_existing(&self, path: &VaultPath) -> Result<PathBuf> {
        let absolute = self.resolve_for_write(path)?;
        let canonical = absolute.canonicalize().map_err(|err| io(&absolute, err))?;

        if !canonical.starts_with(&self.root) {
            return Err(VaultFsError::EscapesVault(path.clone()));
        }

        Ok(canonical)
    }

    pub(crate) fn resolve_for_write(&self, path: &VaultPath) -> Result<PathBuf> {
        let absolute = path.to_path_buf_under(&self.root);
        Ok(absolute)
    }

    fn ensure_write_target_inside(&self, path: &VaultPath, absolute: &Path) -> Result<()> {
        let mut current = absolute.parent();

        while let Some(parent) = current {
            if parent.exists() {
                let canonical = parent.canonicalize().map_err(|err| io(parent, err))?;

                if !canonical.starts_with(&self.root) {
                    return Err(VaultFsError::EscapesVault(path.clone()));
                }

                return Ok(());
            }

            current = parent.parent();
        }

        Err(VaultFsError::EscapesVault(path.clone()))
    }

    pub(crate) fn path_from_absolute(&self, absolute: PathBuf) -> Result<VaultPath> {
        let relative = absolute
            .strip_prefix(&self.root)
            .map_err(|_| VaultFsError::NonUtf8Path(absolute.clone()))?;
        let relative = relative
            .to_str()
            .ok_or_else(|| VaultFsError::NonUtf8Path(absolute.clone()))?;

        Ok(VaultPath::new(relative)?)
    }

    fn apply_times(&self, path: &VaultPath, times: Option<VaultFileTimes>) -> Result<()> {
        let Some(times) = times else {
            return Ok(());
        };

        if let Some(mtime) = times.mtime {
            self.set_mtime(path, mtime)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VaultFileTimes {
    pub ctime: Option<RemoteMillis>,
    pub mtime: Option<RemoteMillis>,
}

impl VaultFileTimes {
    pub fn new(ctime: Option<RemoteMillis>, mtime: Option<RemoteMillis>) -> Self {
        Self { ctime, mtime }
    }
}

fn ensure_parent_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|err| io(parent, err))?;
    }

    Ok(())
}

fn set_modified_time(path: &Path, mtime: RemoteMillis) -> Result<()> {
    let secs = mtime.as_i64() / 1000;
    let nanos = ((mtime.as_i64() % 1000) * 1_000_000) as u32;
    let file_time = FileTime::from_unix_time(secs, nanos);
    filetime::set_file_mtime(path, file_time).map_err(|err| io(path, err))
}
