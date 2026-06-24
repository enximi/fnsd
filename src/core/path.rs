use std::fmt;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::core::{CoreError, Result};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VaultName(String);

impl VaultName {
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();

        if value.is_empty() || value.trim() != value {
            return Err(CoreError::EmptyVaultName);
        }

        if value
            .chars()
            .any(|ch| ch == '/' || ch == '\\' || ch.is_control())
        {
            return Err(CoreError::InvalidVaultName);
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for VaultName {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for VaultName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for VaultName {
    type Err = CoreError;

    fn from_str(value: &str) -> Result<Self> {
        Self::new(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VaultPath(String);

impl VaultPath {
    pub fn new(value: impl AsRef<str>) -> Result<Self> {
        normalize_vault_path(value.as_ref()).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }

    pub fn to_path_buf_under(&self, root: impl AsRef<Path>) -> PathBuf {
        let mut path = root.as_ref().to_path_buf();

        for segment in self.0.split('/') {
            path.push(segment);
        }

        path
    }
}

impl AsRef<str> for VaultPath {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for VaultPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for VaultPath {
    type Err = CoreError;

    fn from_str(value: &str) -> Result<Self> {
        Self::new(value)
    }
}

fn normalize_vault_path(value: &str) -> Result<String> {
    if value.is_empty() || value.trim() != value {
        return Err(CoreError::EmptyPath);
    }

    if value.chars().any(|ch| ch == '\0' || ch.is_control()) {
        return Err(CoreError::InvalidPath);
    }

    let value = value.replace('\\', "/");

    if value.starts_with('/') || value.starts_with("//") || has_windows_drive_prefix(&value) {
        return Err(CoreError::AbsolutePath);
    }

    let mut segments = Vec::new();

    for segment in value.split('/') {
        match segment {
            "" | "." => {}
            ".." => return Err(CoreError::ParentTraversal),
            segment if segment.ends_with(':') => return Err(CoreError::InvalidPath),
            segment => segments.push(segment),
        }
    }

    if segments.is_empty() {
        return Err(CoreError::EmptyPath);
    }

    Ok(segments.join("/"))
}

fn has_windows_drive_prefix(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':'
}
