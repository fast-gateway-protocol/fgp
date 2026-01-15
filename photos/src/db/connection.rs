//! SQLite connection management for Photos library database.
//!
//! CHANGELOG:
//! - 01/15/2026 - Initial implementation (Claude)

use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::PathBuf;

/// Get the Photos library path.
pub fn photos_library_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Pictures")
        .join("Photos Library.photoslibrary")
}

/// Default Photos database path.
pub fn photos_db_path() -> PathBuf {
    photos_library_path()
        .join("database")
        .join("Photos.sqlite")
}

/// Open a read-only connection to Photos database.
pub fn open_photos_db() -> Result<Connection> {
    let db_path = photos_db_path();

    Connection::open_with_flags(
        &db_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .with_context(|| format!("Failed to open Photos database at {:?}. Full Disk Access may be required.", db_path))
}

/// Check if we have access to Photos database.
pub fn check_access() -> bool {
    open_photos_db().is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_photos_db_path() {
        let path = photos_db_path();
        assert!(path.ends_with("Photos Library.photoslibrary/database/Photos.sqlite"));
    }
}
