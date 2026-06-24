use crate::core::{ContentHash, ResourceKind, VaultPath};
use crate::store::{PendingRename, Result, UploadCheckpoint};
use rusqlite::{Connection, OptionalExtension, params};

use super::kind_key;

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
