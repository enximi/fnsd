use crate::core::{RemoteMillis, ResourceKind, VaultPath};
use crate::store::{HashEntry, Result};
use rusqlite::{Connection, OptionalExtension, params};

use super::{all_resource_kinds, kind_key, to_i64};

pub(crate) fn sync_time(conn: &Connection, kind: ResourceKind) -> Result<RemoteMillis> {
    let value = conn
        .query_row(
            "SELECT value FROM sync_times WHERE kind = ?1",
            params![kind_key(kind)],
            |row| row.get::<_, i64>(0),
        )
        .optional()?
        .unwrap_or_default();
    Ok(RemoteMillis::new(value)?)
}

pub(crate) fn set_sync_time(
    conn: &Connection,
    kind: ResourceKind,
    value: RemoteMillis,
) -> Result<()> {
    conn.execute(
        "
        INSERT INTO sync_times (kind, value)
        VALUES (?1, ?2)
        ON CONFLICT(kind) DO UPDATE SET value = excluded.value
        ",
        params![kind_key(kind), value.as_i64()],
    )?;
    Ok(())
}

pub(crate) fn hash_entry(
    conn: &Connection,
    kind: ResourceKind,
    path: &VaultPath,
) -> Result<Option<HashEntry>> {
    conn.query_row(
        "
        SELECT content_hash, mtime, size
        FROM hash_entries
        WHERE kind = ?1 AND path = ?2
        ",
        params![kind_key(kind), path.as_str()],
        |row| {
            Ok(HashEntry {
                content_hash: row.get(0)?,
                mtime: row.get(1)?,
                size: row.get::<_, i64>(2).and_then(size_from_i64)?,
            })
        },
    )
    .optional()
    .map_err(Into::into)
}

pub(crate) fn set_hash_entry(
    conn: &Connection,
    kind: ResourceKind,
    path: &VaultPath,
    entry: &HashEntry,
) -> Result<()> {
    conn.execute(
        "
        INSERT INTO hash_entries (kind, path, content_hash, mtime, size)
        VALUES (?1, ?2, ?3, ?4, ?5)
        ON CONFLICT(kind, path) DO UPDATE SET
            content_hash = excluded.content_hash,
            mtime = excluded.mtime,
            size = excluded.size
        ",
        params![
            kind_key(kind),
            path.as_str(),
            entry.content_hash.as_deref(),
            entry.mtime,
            to_i64("size", entry.size)?,
        ],
    )?;
    Ok(())
}

pub(crate) fn remove_hash_entry(
    conn: &Connection,
    kind: ResourceKind,
    path: &VaultPath,
) -> Result<Option<HashEntry>> {
    let entry = hash_entry(conn, kind, path)?;
    conn.execute(
        "DELETE FROM hash_entries WHERE kind = ?1 AND path = ?2",
        params![kind_key(kind), path.as_str()],
    )?;
    Ok(entry)
}

pub(crate) fn rename_hash_tree(
    conn: &mut Connection,
    old_path: &VaultPath,
    new_path: &VaultPath,
) -> Result<()> {
    let tx = conn.transaction()?;
    for kind in all_resource_kinds() {
        let rows = hash_tree_rows(&tx, kind, old_path.as_str())?;
        for (path, _) in &rows {
            tx.execute(
                "DELETE FROM hash_entries WHERE kind = ?1 AND path = ?2",
                params![kind_key(kind), path],
            )?;
        }
        for (path, entry) in rows {
            let target = renamed_path(&path, old_path.as_str(), new_path.as_str());
            tx.execute(
                "
                INSERT INTO hash_entries (kind, path, content_hash, mtime, size)
                VALUES (?1, ?2, ?3, ?4, ?5)
                ON CONFLICT(kind, path) DO UPDATE SET
                    content_hash = excluded.content_hash,
                    mtime = excluded.mtime,
                    size = excluded.size
                ",
                params![
                    kind_key(kind),
                    target,
                    entry.content_hash.as_deref(),
                    entry.mtime,
                    to_i64("size", entry.size)?,
                ],
            )?;
        }
    }
    tx.commit()?;
    Ok(())
}

pub(crate) fn all_hash_paths(conn: &Connection, kind: ResourceKind) -> Result<Vec<VaultPath>> {
    let mut stmt = conn.prepare("SELECT path FROM hash_entries WHERE kind = ?1 ORDER BY path")?;
    let rows = stmt.query_map(params![kind_key(kind)], |row| row.get::<_, String>(0))?;
    rows.map(|row| Ok(VaultPath::new(&row?)?)).collect()
}

fn hash_tree_rows(
    conn: &Connection,
    kind: ResourceKind,
    old_path: &str,
) -> Result<Vec<(String, HashEntry)>> {
    let old_prefix = format!("{old_path}/");
    let mut stmt = conn.prepare(
        "
        SELECT path, content_hash, mtime, size
        FROM hash_entries
        WHERE kind = ?1
        ORDER BY path
        ",
    )?;
    let rows = stmt.query_map(params![kind_key(kind)], |row| {
        Ok((
            row.get::<_, String>(0)?,
            HashEntry {
                content_hash: row.get(1)?,
                mtime: row.get(2)?,
                size: row.get::<_, i64>(3).and_then(size_from_i64)?,
            },
        ))
    })?;

    let mut matches = Vec::new();
    for row in rows {
        let (path, entry) = row?;
        if path == old_path || path.starts_with(&old_prefix) {
            matches.push((path, entry));
        }
    }
    Ok(matches)
}

fn renamed_path(path: &str, old_path: &str, new_path: &str) -> String {
    if path == old_path {
        return new_path.to_string();
    }

    let old_prefix = format!("{old_path}/");
    let new_prefix = format!("{new_path}/");
    let suffix = path
        .strip_prefix(&old_prefix)
        .expect("hash tree query only returns matching descendants");
    format!("{new_prefix}{suffix}")
}

fn size_from_i64(value: i64) -> rusqlite::Result<u64> {
    value
        .try_into()
        .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(0, value))
}
