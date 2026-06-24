use std::fmt;
use std::str::FromStr;

use crate::core::{CoreError, Result};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PathHash(String);

impl PathHash {
    pub fn new(value: impl Into<String>) -> Result<Self> {
        validate_hash(value.into()).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for PathHash {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for PathHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for PathHash {
    type Err = CoreError;

    fn from_str(value: &str) -> Result<Self> {
        Self::new(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContentHash(String);

impl ContentHash {
    pub fn new(value: impl Into<String>) -> Result<Self> {
        validate_hash(value.into()).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl AsRef<str> for ContentHash {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for ContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for ContentHash {
    type Err = CoreError;

    fn from_str(value: &str) -> Result<Self> {
        Self::new(value)
    }
}

fn validate_hash(value: String) -> Result<String> {
    if value.is_empty() || value.trim() != value {
        return Err(CoreError::InvalidHash);
    }

    if value
        .chars()
        .any(|ch| ch.is_control() || ch.is_whitespace())
    {
        return Err(CoreError::InvalidHash);
    }

    Ok(value)
}
