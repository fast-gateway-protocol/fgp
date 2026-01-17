//! Notes database queries.
//!
//! CHANGELOG:
//! - 01/15/2026 - Initial implementation (Claude)

use anyhow::Result;
use chrono::{TimeZone, Utc};
use flate2::read::GzDecoder;
use prost::Message;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::io::Read;

// Include generated protobuf code
pub mod notestore {
    include!(concat!(env!("OUT_DIR"), "/notestore.rs"));
}

/// Core Data epoch (2001-01-01 00:00:00 UTC)
const CORE_DATA_EPOCH: i64 = 978307200;

/// Convert Core Data timestamp to ISO string.
fn core_data_to_iso(timestamp: f64) -> String {
    let unix_timestamp = (timestamp as i64) + CORE_DATA_EPOCH;
    Utc.timestamp_opt(unix_timestamp, 0)
        .single()
        .map(|dt| dt.to_rfc3339())
        .unwrap_or_default()
}

/// A note from the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteRecord {
    pub id: i64,
    pub title: Option<String>,
    pub snippet: Option<String>,
    pub body: Option<String>,
    pub created: Option<String>,
    pub modified: Option<String>,
    pub folder: Option<String>,
    pub is_pinned: bool,
    pub has_checklist: bool,
}

/// A folder from the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderRecord {
    pub id: i64,
    pub name: String,
    pub note_count: i64,
}

/// Library statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotesStats {
    pub total_notes: i64,
    pub total_folders: i64,
    pub notes_with_checklists: i64,
    pub pinned_notes: i64,
}

/// Extract plain text from gzipped protobuf data.
pub fn extract_note_text(data: &[u8]) -> Option<String> {
    // Check for gzip magic bytes
    if data.len() < 2 || data[0] != 0x1F || data[1] != 0x8B {
        return None;
    }

    // Decompress gzip
    let mut decoder = GzDecoder::new(data);
    let mut decompressed = Vec::new();
    if decoder.read_to_end(&mut decompressed).is_err() {
        return None;
    }

    // Parse protobuf
    match notestore::NoteStoreProto::decode(decompressed.as_slice()) {
        Ok(proto) => {
            // All fields are required in proto2, access directly
            Some(proto.document.note.note_text)
        }
        Err(_) => None,
    }
}

/// Query recent notes.
pub fn query_recent_notes(conn: &Connection, days: u32, limit: u32) -> Result<Vec<NoteRecord>> {
    let cutoff = Utc::now().timestamp() - CORE_DATA_EPOCH - (days as i64 * 86400);

    let mut stmt = conn.prepare(
        "SELECT
            n.Z_PK,
            n.ZTITLE1,
            n.ZSNIPPET,
            n.ZCREATIONDATE3,
            n.ZMODIFICATIONDATE1,
            n.ZISPINNED,
            n.ZHASCHECKLIST,
            d.ZDATA,
            f.ZTITLE2 as folder_name
         FROM ZICCLOUDSYNCINGOBJECT n
         LEFT JOIN ZICNOTEDATA d ON d.ZNOTE = n.Z_PK
         LEFT JOIN ZICCLOUDSYNCINGOBJECT f ON n.ZFOLDER = f.Z_PK
         WHERE n.Z_ENT = 12
           AND n.ZMARKEDFORDELETION = 0
           AND n.ZMODIFICATIONDATE1 > ?
         ORDER BY n.ZMODIFICATIONDATE1 DESC
         LIMIT ?",
    )?;

    let notes = stmt
        .query_map([cutoff as f64, limit as f64], |row| {
            let data: Option<Vec<u8>> = row.get(7)?;
            let body = data.as_ref().and_then(|d| extract_note_text(d));

            Ok(NoteRecord {
                id: row.get(0)?,
                title: row.get(1)?,
                snippet: row.get(2)?,
                body,
                created: row
                    .get::<_, Option<f64>>(3)?
                    .map(|ts| core_data_to_iso(ts)),
                modified: row
                    .get::<_, Option<f64>>(4)?
                    .map(|ts| core_data_to_iso(ts)),
                is_pinned: row.get::<_, Option<i32>>(5)?.unwrap_or(0) == 1,
                has_checklist: row.get::<_, Option<i32>>(6)?.unwrap_or(0) == 1,
                folder: row.get(8)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(notes)
}

/// Query all notes.
pub fn query_notes(conn: &Connection, limit: u32) -> Result<Vec<NoteRecord>> {
    let mut stmt = conn.prepare(
        "SELECT
            n.Z_PK,
            n.ZTITLE1,
            n.ZSNIPPET,
            n.ZCREATIONDATE3,
            n.ZMODIFICATIONDATE1,
            n.ZISPINNED,
            n.ZHASCHECKLIST,
            d.ZDATA,
            f.ZTITLE2 as folder_name
         FROM ZICCLOUDSYNCINGOBJECT n
         LEFT JOIN ZICNOTEDATA d ON d.ZNOTE = n.Z_PK
         LEFT JOIN ZICCLOUDSYNCINGOBJECT f ON n.ZFOLDER = f.Z_PK
         WHERE n.Z_ENT = 12
           AND n.ZMARKEDFORDELETION = 0
         ORDER BY n.ZMODIFICATIONDATE1 DESC
         LIMIT ?",
    )?;

    let notes = stmt
        .query_map([limit], |row| {
            let data: Option<Vec<u8>> = row.get(7)?;
            let body = data.as_ref().and_then(|d| extract_note_text(d));

            Ok(NoteRecord {
                id: row.get(0)?,
                title: row.get(1)?,
                snippet: row.get(2)?,
                body,
                created: row
                    .get::<_, Option<f64>>(3)?
                    .map(|ts| core_data_to_iso(ts)),
                modified: row
                    .get::<_, Option<f64>>(4)?
                    .map(|ts| core_data_to_iso(ts)),
                is_pinned: row.get::<_, Option<i32>>(5)?.unwrap_or(0) == 1,
                has_checklist: row.get::<_, Option<i32>>(6)?.unwrap_or(0) == 1,
                folder: row.get(8)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(notes)
}

/// Search notes by title/content.
pub fn search_notes(conn: &Connection, query: &str, limit: u32) -> Result<Vec<NoteRecord>> {
    let search_pattern = format!("%{}%", query);

    let mut stmt = conn.prepare(
        "SELECT
            n.Z_PK,
            n.ZTITLE1,
            n.ZSNIPPET,
            n.ZCREATIONDATE3,
            n.ZMODIFICATIONDATE1,
            n.ZISPINNED,
            n.ZHASCHECKLIST,
            d.ZDATA,
            f.ZTITLE2 as folder_name
         FROM ZICCLOUDSYNCINGOBJECT n
         LEFT JOIN ZICNOTEDATA d ON d.ZNOTE = n.Z_PK
         LEFT JOIN ZICCLOUDSYNCINGOBJECT f ON n.ZFOLDER = f.Z_PK
         WHERE n.Z_ENT = 12
           AND n.ZMARKEDFORDELETION = 0
           AND (n.ZTITLE1 LIKE ? OR n.ZSNIPPET LIKE ?)
         ORDER BY n.ZMODIFICATIONDATE1 DESC
         LIMIT ?",
    )?;

    let notes = stmt
        .query_map([&search_pattern, &search_pattern, &limit.to_string()], |row| {
            let data: Option<Vec<u8>> = row.get(7)?;
            let body = data.as_ref().and_then(|d| extract_note_text(d));

            Ok(NoteRecord {
                id: row.get(0)?,
                title: row.get(1)?,
                snippet: row.get(2)?,
                body,
                created: row
                    .get::<_, Option<f64>>(3)?
                    .map(|ts| core_data_to_iso(ts)),
                modified: row
                    .get::<_, Option<f64>>(4)?
                    .map(|ts| core_data_to_iso(ts)),
                is_pinned: row.get::<_, Option<i32>>(5)?.unwrap_or(0) == 1,
                has_checklist: row.get::<_, Option<i32>>(6)?.unwrap_or(0) == 1,
                folder: row.get(8)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(notes)
}

/// Get a specific note by ID.
pub fn get_note(conn: &Connection, note_id: i64) -> Result<Option<NoteRecord>> {
    let mut stmt = conn.prepare(
        "SELECT
            n.Z_PK,
            n.ZTITLE1,
            n.ZSNIPPET,
            n.ZCREATIONDATE3,
            n.ZMODIFICATIONDATE1,
            n.ZISPINNED,
            n.ZHASCHECKLIST,
            d.ZDATA,
            f.ZTITLE2 as folder_name
         FROM ZICCLOUDSYNCINGOBJECT n
         LEFT JOIN ZICNOTEDATA d ON d.ZNOTE = n.Z_PK
         LEFT JOIN ZICCLOUDSYNCINGOBJECT f ON n.ZFOLDER = f.Z_PK
         WHERE n.Z_PK = ?
           AND n.Z_ENT = 12",
    )?;

    let note = stmt
        .query_row([note_id], |row| {
            let data: Option<Vec<u8>> = row.get(7)?;
            let body = data.as_ref().and_then(|d| extract_note_text(d));

            Ok(NoteRecord {
                id: row.get(0)?,
                title: row.get(1)?,
                snippet: row.get(2)?,
                body,
                created: row
                    .get::<_, Option<f64>>(3)?
                    .map(|ts| core_data_to_iso(ts)),
                modified: row
                    .get::<_, Option<f64>>(4)?
                    .map(|ts| core_data_to_iso(ts)),
                is_pinned: row.get::<_, Option<i32>>(5)?.unwrap_or(0) == 1,
                has_checklist: row.get::<_, Option<i32>>(6)?.unwrap_or(0) == 1,
                folder: row.get(8)?,
            })
        })
        .ok();

    Ok(note)
}

/// Get notes in a specific folder.
pub fn query_folder_notes(
    conn: &Connection,
    folder_name: &str,
    limit: u32,
) -> Result<Vec<NoteRecord>> {
    let mut stmt = conn.prepare(
        "SELECT
            n.Z_PK,
            n.ZTITLE1,
            n.ZSNIPPET,
            n.ZCREATIONDATE3,
            n.ZMODIFICATIONDATE1,
            n.ZISPINNED,
            n.ZHASCHECKLIST,
            d.ZDATA,
            f.ZTITLE2 as folder_name
         FROM ZICCLOUDSYNCINGOBJECT n
         LEFT JOIN ZICNOTEDATA d ON d.ZNOTE = n.Z_PK
         JOIN ZICCLOUDSYNCINGOBJECT f ON n.ZFOLDER = f.Z_PK
         WHERE n.Z_ENT = 12
           AND n.ZMARKEDFORDELETION = 0
           AND f.ZTITLE2 LIKE ?
         ORDER BY n.ZMODIFICATIONDATE1 DESC
         LIMIT ?",
    )?;

    let folder_pattern = format!("%{}%", folder_name);

    let notes = stmt
        .query_map([&folder_pattern, &limit.to_string()], |row| {
            let data: Option<Vec<u8>> = row.get(7)?;
            let body = data.as_ref().and_then(|d| extract_note_text(d));

            Ok(NoteRecord {
                id: row.get(0)?,
                title: row.get(1)?,
                snippet: row.get(2)?,
                body,
                created: row
                    .get::<_, Option<f64>>(3)?
                    .map(|ts| core_data_to_iso(ts)),
                modified: row
                    .get::<_, Option<f64>>(4)?
                    .map(|ts| core_data_to_iso(ts)),
                is_pinned: row.get::<_, Option<i32>>(5)?.unwrap_or(0) == 1,
                has_checklist: row.get::<_, Option<i32>>(6)?.unwrap_or(0) == 1,
                folder: row.get(8)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(notes)
}

/// Query pinned notes.
pub fn query_pinned_notes(conn: &Connection, limit: u32) -> Result<Vec<NoteRecord>> {
    let mut stmt = conn.prepare(
        "SELECT
            n.Z_PK,
            n.ZTITLE1,
            n.ZSNIPPET,
            n.ZCREATIONDATE3,
            n.ZMODIFICATIONDATE1,
            n.ZISPINNED,
            n.ZHASCHECKLIST,
            d.ZDATA,
            f.ZTITLE2 as folder_name
         FROM ZICCLOUDSYNCINGOBJECT n
         LEFT JOIN ZICNOTEDATA d ON d.ZNOTE = n.Z_PK
         LEFT JOIN ZICCLOUDSYNCINGOBJECT f ON n.ZFOLDER = f.Z_PK
         WHERE n.Z_ENT = 12
           AND n.ZMARKEDFORDELETION = 0
           AND n.ZISPINNED = 1
         ORDER BY n.ZMODIFICATIONDATE1 DESC
         LIMIT ?",
    )?;

    let notes = stmt
        .query_map([limit], |row| {
            let data: Option<Vec<u8>> = row.get(7)?;
            let body = data.as_ref().and_then(|d| extract_note_text(d));

            Ok(NoteRecord {
                id: row.get(0)?,
                title: row.get(1)?,
                snippet: row.get(2)?,
                body,
                created: row
                    .get::<_, Option<f64>>(3)?
                    .map(|ts| core_data_to_iso(ts)),
                modified: row
                    .get::<_, Option<f64>>(4)?
                    .map(|ts| core_data_to_iso(ts)),
                is_pinned: true,
                has_checklist: row.get::<_, Option<i32>>(6)?.unwrap_or(0) == 1,
                folder: row.get(8)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(notes)
}

/// Query folders.
pub fn query_folders(conn: &Connection, limit: u32) -> Result<Vec<FolderRecord>> {
    let mut stmt = conn.prepare(
        "SELECT
            f.Z_PK,
            f.ZTITLE2,
            COUNT(n.Z_PK) as note_count
         FROM ZICCLOUDSYNCINGOBJECT f
         LEFT JOIN ZICCLOUDSYNCINGOBJECT n ON n.ZFOLDER = f.Z_PK AND n.Z_ENT = 12 AND n.ZMARKEDFORDELETION = 0
         WHERE f.Z_ENT = 9
         GROUP BY f.Z_PK
         ORDER BY note_count DESC
         LIMIT ?",
    )?;

    let folders = stmt
        .query_map([limit], |row| {
            Ok(FolderRecord {
                id: row.get(0)?,
                name: row.get::<_, Option<String>>(1)?.unwrap_or_else(|| "Notes".to_string()),
                note_count: row.get(2)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(folders)
}

/// Get library statistics.
pub fn query_stats(conn: &Connection) -> Result<NotesStats> {
    let total_notes: i64 = conn.query_row(
        "SELECT COUNT(*) FROM ZICCLOUDSYNCINGOBJECT WHERE Z_ENT = 12 AND ZMARKEDFORDELETION = 0",
        [],
        |row| row.get(0),
    )?;

    let total_folders: i64 = conn.query_row(
        "SELECT COUNT(*) FROM ZICCLOUDSYNCINGOBJECT WHERE Z_ENT = 9",
        [],
        |row| row.get(0),
    )?;

    let notes_with_checklists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM ZICCLOUDSYNCINGOBJECT WHERE Z_ENT = 12 AND ZMARKEDFORDELETION = 0 AND ZHASCHECKLIST = 1",
        [],
        |row| row.get(0),
    )?;

    let pinned_notes: i64 = conn.query_row(
        "SELECT COUNT(*) FROM ZICCLOUDSYNCINGOBJECT WHERE Z_ENT = 12 AND ZMARKEDFORDELETION = 0 AND ZISPINNED = 1",
        [],
        |row| row.get(0),
    )?;

    Ok(NotesStats {
        total_notes,
        total_folders,
        notes_with_checklists,
        pinned_notes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup_notes_db() -> Connection {
        let conn = Connection::open_in_memory().expect("open memory db");
        conn.execute_batch(
            r#"
            CREATE TABLE ZICCLOUDSYNCINGOBJECT (
                Z_PK INTEGER PRIMARY KEY,
                ZTITLE1 TEXT,
                ZSNIPPET TEXT,
                ZCREATIONDATE3 REAL,
                ZMODIFICATIONDATE1 REAL,
                ZISPINNED INTEGER,
                ZHASCHECKLIST INTEGER,
                ZFOLDER INTEGER,
                ZENT INTEGER,
                Z_ENT INTEGER,
                ZMARKEDFORDELETION INTEGER
            );
            CREATE TABLE ZICNOTEDATA (
                ZNOTE INTEGER,
                ZDATA BLOB
            );
            "#,
        )
        .expect("create schema");
        conn
    }

    #[test]
    fn test_extract_note_text_requires_gzip() {
        let data = b"not-gzip";
        assert!(extract_note_text(data).is_none());
    }

    #[test]
    fn test_query_stats_counts() {
        let conn = setup_notes_db();
        conn.execute(
            "INSERT INTO ZICCLOUDSYNCINGOBJECT (Z_PK, Z_ENT, ZMARKEDFORDELETION, ZHASCHECKLIST, ZISPINNED) VALUES (1, 12, 0, 1, 1)",
            [],
        )
        .expect("insert note");
        conn.execute(
            "INSERT INTO ZICCLOUDSYNCINGOBJECT (Z_PK, Z_ENT, ZMARKEDFORDELETION, ZHASCHECKLIST, ZISPINNED) VALUES (2, 12, 0, 0, 0)",
            [],
        )
        .expect("insert note");
        conn.execute(
            "INSERT INTO ZICCLOUDSYNCINGOBJECT (Z_PK, Z_ENT, ZMARKEDFORDELETION) VALUES (3, 9, 0)",
            [],
        )
        .expect("insert folder");

        let stats = query_stats(&conn).expect("stats");

        assert_eq!(stats.total_notes, 2);
        assert_eq!(stats.total_folders, 1);
        assert_eq!(stats.notes_with_checklists, 1);
        assert_eq!(stats.pinned_notes, 1);
    }
}
