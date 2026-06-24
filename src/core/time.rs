use std::fmt;

use crate::core::{CoreError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RemoteMillis(i64);

impl RemoteMillis {
    pub fn new(value: i64) -> Result<Self> {
        if value < 0 {
            return Err(CoreError::InvalidTimestamp);
        }

        Ok(Self(value))
    }

    pub fn as_i64(self) -> i64 {
        self.0
    }
}

impl TryFrom<i64> for RemoteMillis {
    type Error = CoreError;

    fn try_from(value: i64) -> Result<Self> {
        Self::new(value)
    }
}

impl From<RemoteMillis> for i64 {
    fn from(value: RemoteMillis) -> Self {
        value.0
    }
}

impl fmt::Display for RemoteMillis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
