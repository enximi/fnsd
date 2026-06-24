use crate::core::ContentHash;
use crate::store::Result;
use crate::sync::transfer::DownloadSession;
use rusqlite::{Connection, params};

use super::to_i64;

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
