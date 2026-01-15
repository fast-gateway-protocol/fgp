//! Database access layer for Safari databases.
//!
//! Safari stores data in multiple SQLite databases:
//! - History.db - Browser history and visits
//! - CloudTabs.db - Tabs synced from other devices
//! - Bookmarks.plist - Bookmarks (XML plist, not SQLite)
//!
//! CHANGELOG:
//! - 01/15/2026 - Initial implementation (Claude)

pub mod connection;
pub mod queries;
