//! Notes database connection handling.
//!
//! CHANGELOG:
//! - 01/15/2026 - Initial implementation (Claude)

use anyhow::{anyhow, Result};
use rusqlite::Connection;
use std::path::PathBuf;

/// Get the Notes database path.
pub fn notes_db_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/Users".to_string());
    PathBuf::from(home)
        .join("Library/Group Containers/group.com.apple.notes/NoteStore.sqlite")
}

/// Open the Notes database in read-only mode.
pub fn open_notes_db() -> Result<Connection> {
    let db_path = notes_db_path();

    if !db_path.exists() {
        return Err(anyhow!(
            "Notes database not found at {}. Make sure you have Full Disk Access enabled.",
            db_path.display()
        ));
    }

    let conn = Connection::open_with_flags(
        &db_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;

    // Note: WAL mode enables concurrent reads but requires write access to set.
    // Apple Notes already uses WAL, so we benefit from it without needing to set it.
    // Just verify we can query the database.
    conn.pragma_query(None, "journal_mode", |_row| Ok(()))?;

    Ok(conn)
}
