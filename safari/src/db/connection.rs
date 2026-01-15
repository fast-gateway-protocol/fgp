//! SQLite connection management for Safari databases.
//!
//! CHANGELOG:
//! - 01/15/2026 - Initial implementation (Claude)

use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::PathBuf;

/// Get the Safari library directory path.
pub fn safari_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Library")
        .join("Safari")
}

/// Default History.db path.
pub fn history_db_path() -> PathBuf {
    safari_dir().join("History.db")
}

/// Default CloudTabs.db path.
pub fn cloud_tabs_db_path() -> PathBuf {
    safari_dir().join("CloudTabs.db")
}

/// Default Bookmarks.plist path.
pub fn bookmarks_plist_path() -> PathBuf {
    safari_dir().join("Bookmarks.plist")
}

/// Open a read-only connection to History.db.
pub fn open_history_db() -> Result<Connection> {
    let db_path = history_db_path();

    Connection::open_with_flags(
        &db_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .with_context(|| format!("Failed to open Safari History database at {:?}", db_path))
}

/// Open a read-only connection to CloudTabs.db.
pub fn open_cloud_tabs_db() -> Result<Connection> {
    let db_path = cloud_tabs_db_path();

    Connection::open_with_flags(
        &db_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .with_context(|| format!("Failed to open Safari CloudTabs database at {:?}", db_path))
}

/// Check if we have access to Safari History database.
pub fn check_history_access() -> bool {
    open_history_db().is_ok()
}

/// Check if we have access to Safari CloudTabs database.
pub fn check_cloud_tabs_access() -> bool {
    open_cloud_tabs_db().is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_db_path() {
        let path = history_db_path();
        assert!(path.ends_with("Library/Safari/History.db"));
    }

    #[test]
    fn test_cloud_tabs_db_path() {
        let path = cloud_tabs_db_path();
        assert!(path.ends_with("Library/Safari/CloudTabs.db"));
    }
}
