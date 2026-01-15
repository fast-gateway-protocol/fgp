//! SQLite connection management for AddressBook database.
//!
//! CHANGELOG:
//! - 01/15/2026 - Initial implementation (Claude)

use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::PathBuf;

/// Get the AddressBook directory path.
pub fn addressbook_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Library")
        .join("Application Support")
        .join("AddressBook")
}

/// Default AddressBook database path.
pub fn addressbook_db_path() -> PathBuf {
    addressbook_dir().join("AddressBook-v22.abcddb")
}

/// Open a read-only connection to AddressBook database.
pub fn open_addressbook_db() -> Result<Connection> {
    let db_path = addressbook_db_path();

    Connection::open_with_flags(
        &db_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .with_context(|| format!("Failed to open AddressBook database at {:?}. Full Disk Access may be required.", db_path))
}

/// Check if we have access to AddressBook database.
pub fn check_access() -> bool {
    open_addressbook_db().is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_addressbook_db_path() {
        let path = addressbook_db_path();
        assert!(path.ends_with("AddressBook/AddressBook-v22.abcddb"));
    }
}
