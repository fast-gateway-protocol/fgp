//! Database access layer for Photos library.
//!
//! macOS Photos stores data in a Core Data SQLite database at:
//! ~/Pictures/Photos Library.photoslibrary/database/Photos.sqlite
//!
//! Key tables:
//! - ZASSET: Main asset records (photos, videos)
//! - ZGENERICALBUM: Albums and folders
//! - Z_33ASSETS: Album-asset relationships
//! - ZPERSON: Recognized people
//! - ZDETECTEDFACE: Face detection data
//!
//! CHANGELOG:
//! - 01/15/2026 - Initial implementation (Claude)

pub mod connection;
pub mod queries;
