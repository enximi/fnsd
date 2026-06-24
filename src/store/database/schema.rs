use crate::store::Result;
use rusqlite::Connection;

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
