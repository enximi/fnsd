use fns_core::{PathHash, Result, VaultPath};

use crate::text::text_hash_value;

pub fn path_hash(path: impl AsRef<str>) -> Result<PathHash> {
    let path = VaultPath::new(path.as_ref())?;
    PathHash::new(text_hash_value(path.as_str()))
}
