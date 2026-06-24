use crate::core::{ContentHash, RemoteMillis, ResourceKind, VaultPath};
use crate::store::{HashEntry, PendingRename, Result, UploadCheckpoint};
use crate::sync::transfer::DownloadSession;
use rusqlite::{Connection, OptionalExtension, params};

pub(crate) fn initialize_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS metadata (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS sync_times (
            kind TEXT PRIMARY KEY,
            value INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS hash_entries (
            kind TEXT NOT NULL,
            path TEXT NOT NULL,
            content_hash TEXT,
            mtime INTEGER NOT NULL,
            size INTEGER NOT NULL,
            PRIMARY KEY (kind, path)
        );

        CREATE TABLE IF NOT EXISTS pending_modifies (
            kind TEXT NOT NULL,
            path TEXT NOT NULL,
            content_hash TEXT NOT NULL,
            PRIMARY KEY (kind, path)
        );

        CREATE TABLE IF NOT EXISTS pending_deletes (
            kind TEXT NOT NULL,
            path TEXT NOT NULL,
            PRIMARY KEY (kind, path)
        );

        CREATE TABLE IF NOT EXISTS pending_renames (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            kind TEXT NOT NULL,
            old_path TEXT NOT NULL,
            new_path TEXT NOT NULL,
            content_hash TEXT
        );

        CREATE TABLE IF NOT EXISTS upload_checkpoints (
            path TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            content_hash TEXT NOT NULL,
            last_chunk_index INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS download_chunks (
            content_hash TEXT NOT NULL,
            size INTEGER NOT NULL,
            chunk_size INTEGER NOT NULL,
            chunk_index INTEGER NOT NULL,
            data BLOB NOT NULL,
            PRIMARY KEY (content_hash, size, chunk_size, chunk_index)
        );
        ",
    )?;
    conn.execute(
        "INSERT OR IGNORE INTO metadata (key, value) VALUES ('schema_version', '1')",
        [],
    )?;
    Ok(())
}

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
                size: row.get(2)?,
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

pub(crate) fn set_pending_modify(
    conn: &Connection,
    kind: ResourceKind,
    path: &VaultPath,
    content_hash: &ContentHash,
) -> Result<()> {
    if kind == ResourceKind::Folder {
        return Ok(());
    }

    conn.execute(
        "
        INSERT INTO pending_modifies (kind, path, content_hash)
        VALUES (?1, ?2, ?3)
        ON CONFLICT(kind, path) DO UPDATE SET content_hash = excluded.content_hash
        ",
        params![kind_key(kind), path.as_str(), content_hash.as_str()],
    )?;
    Ok(())
}

pub(crate) fn remove_pending_modify(
    conn: &Connection,
    kind: ResourceKind,
    path: &VaultPath,
) -> Result<Option<String>> {
    let content_hash = conn
        .query_row(
            "
            SELECT content_hash
            FROM pending_modifies
            WHERE kind = ?1 AND path = ?2
            ",
            params![kind_key(kind), path.as_str()],
            |row| row.get::<_, String>(0),
        )
        .optional()?;
    conn.execute(
        "DELETE FROM pending_modifies WHERE kind = ?1 AND path = ?2",
        params![kind_key(kind), path.as_str()],
    )?;
    Ok(content_hash)
}

pub(crate) fn has_pending_modify(
    conn: &Connection,
    kind: ResourceKind,
    path: &VaultPath,
) -> Result<bool> {
    let exists = conn
        .query_row(
            "
            SELECT 1
            FROM pending_modifies
            WHERE kind = ?1 AND path = ?2
            LIMIT 1
            ",
            params![kind_key(kind), path.as_str()],
            |_| Ok(()),
        )
        .optional()?
        .is_some();
    Ok(exists)
}

pub(crate) fn file_upload_checkpoint(
    conn: &Connection,
    path: &VaultPath,
) -> Result<Option<UploadCheckpoint>> {
    conn.query_row(
        "
        SELECT session_id, content_hash, last_chunk_index
        FROM upload_checkpoints
        WHERE path = ?1
        ",
        params![path.as_str()],
        |row| {
            Ok(UploadCheckpoint {
                session_id: row.get(0)?,
                content_hash: row.get(1)?,
                last_chunk_index: row.get(2)?,
            })
        },
    )
    .optional()
    .map_err(Into::into)
}

pub(crate) fn set_file_upload_checkpoint(
    conn: &Connection,
    path: &VaultPath,
    checkpoint: &UploadCheckpoint,
) -> Result<()> {
    conn.execute(
        "
        INSERT INTO upload_checkpoints
            (path, session_id, content_hash, last_chunk_index)
        VALUES (?1, ?2, ?3, ?4)
        ON CONFLICT(path) DO UPDATE SET
            session_id = excluded.session_id,
            content_hash = excluded.content_hash,
            last_chunk_index = excluded.last_chunk_index
        ",
        params![
            path.as_str(),
            checkpoint.session_id.as_str(),
            checkpoint.content_hash.as_str(),
            i64::from(checkpoint.last_chunk_index),
        ],
    )?;
    Ok(())
}

pub(crate) fn remove_file_upload_checkpoint(
    conn: &Connection,
    path: &VaultPath,
) -> Result<Option<UploadCheckpoint>> {
    let checkpoint = file_upload_checkpoint(conn, path)?;
    conn.execute(
        "DELETE FROM upload_checkpoints WHERE path = ?1",
        params![path.as_str()],
    )?;
    Ok(checkpoint)
}

pub(crate) fn insert_pending_delete(
    conn: &Connection,
    kind: ResourceKind,
    path: &VaultPath,
) -> Result<()> {
    conn.execute(
        "
        INSERT OR IGNORE INTO pending_deletes (kind, path)
        VALUES (?1, ?2)
        ",
        params![kind_key(kind), path.as_str()],
    )?;
    Ok(())
}

pub(crate) fn remove_pending_delete(
    conn: &Connection,
    kind: ResourceKind,
    path: &VaultPath,
) -> Result<bool> {
    let changed = conn.execute(
        "DELETE FROM pending_deletes WHERE kind = ?1 AND path = ?2",
        params![kind_key(kind), path.as_str()],
    )?;
    Ok(changed > 0)
}

pub(crate) fn push_pending_rename(
    conn: &Connection,
    kind: ResourceKind,
    rename: &PendingRename,
) -> Result<()> {
    if kind == ResourceKind::Setting {
        return Ok(());
    }

    conn.execute(
        "
        INSERT INTO pending_renames (kind, old_path, new_path, content_hash)
        VALUES (?1, ?2, ?3, ?4)
        ",
        params![
            kind_key(kind),
            rename.old_path.as_str(),
            rename.new_path.as_str(),
            rename.content_hash.as_deref(),
        ],
    )?;
    Ok(())
}

pub(crate) fn pop_pending_rename(
    conn: &Connection,
    kind: ResourceKind,
) -> Result<Option<PendingRename>> {
    let row = conn
        .query_row(
            "
            SELECT id, old_path, new_path, content_hash
            FROM pending_renames
            WHERE kind = ?1
            ORDER BY id
            LIMIT 1
            ",
            params![kind_key(kind)],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    PendingRename {
                        old_path: row.get(1)?,
                        new_path: row.get(2)?,
                        content_hash: row.get(3)?,
                    },
                ))
            },
        )
        .optional()?;

    let Some((id, rename)) = row else {
        return Ok(None);
    };

    conn.execute("DELETE FROM pending_renames WHERE id = ?1", params![id])?;
    Ok(Some(rename))
}

pub(crate) fn restore_download_chunks(
    conn: &Connection,
    session: &mut DownloadSession,
) -> Result<()> {
    let mut stmt = conn.prepare(
        "
        SELECT chunk_index, data
        FROM download_chunks
        WHERE content_hash = ?1 AND size = ?2 AND chunk_size = ?3
        ORDER BY chunk_index
        ",
    )?;
    let rows = stmt.query_map(
        params![
            session.content_hash().as_str(),
            to_i64("size", session.size())?,
            to_i64("chunk_size", session.chunk_size() as u64)?,
        ],
        |row| Ok((row.get::<_, i64>(0)?, row.get::<_, Vec<u8>>(1)?)),
    )?;

    for row in rows {
        let (chunk_index, data) = row?;
        session.restore_chunk(
            u32::try_from(chunk_index)
                .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(0, chunk_index))?,
            data,
        )?;
    }

    Ok(())
}

pub(crate) fn save_download_chunk(
    conn: &Connection,
    session: &DownloadSession,
    chunk_index: u32,
    chunk_data: &[u8],
) -> Result<()> {
    conn.execute(
        "
        INSERT OR REPLACE INTO download_chunks
            (content_hash, size, chunk_size, chunk_index, data)
        VALUES (?1, ?2, ?3, ?4, ?5)
        ",
        params![
            session.content_hash().as_str(),
            to_i64("size", session.size())?,
            to_i64("chunk_size", session.chunk_size() as u64)?,
            i64::from(chunk_index),
            chunk_data,
        ],
    )?;
    Ok(())
}

pub(crate) fn clear_download_chunks(
    conn: &Connection,
    content_hash: &ContentHash,
    size: u64,
    chunk_size: usize,
) -> Result<()> {
    conn.execute(
        "
        DELETE FROM download_chunks
        WHERE content_hash = ?1 AND size = ?2 AND chunk_size = ?3
        ",
        params![
            content_hash.as_str(),
            to_i64("size", size)?,
            to_i64("chunk_size", chunk_size as u64)?,
        ],
    )?;
    Ok(())
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
                size: row.get(3)?,
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

fn all_resource_kinds() -> [ResourceKind; 4] {
    [
        ResourceKind::Note,
        ResourceKind::File,
        ResourceKind::Folder,
        ResourceKind::Setting,
    ]
}

fn kind_key(kind: ResourceKind) -> &'static str {
    match kind {
        ResourceKind::Note => "note",
        ResourceKind::File => "file",
        ResourceKind::Folder => "folder",
        ResourceKind::Setting => "setting",
    }
}

fn to_i64(name: &'static str, value: u64) -> Result<i64> {
    i64::try_from(value).map_err(|_| crate::store::LocalStoreError::NumberTooLarge { name, value })
}
