//! Database access layer for AddressBook.
//!
//! macOS stores contacts in a Core Data SQLite database at:
//! ~/Library/Application Support/AddressBook/AddressBook-v22.abcddb
//!
//! Key tables:
//! - ZABCDRECORD: Contact records (names, organization, etc.)
//! - ZABCDEMAILADDRESS: Email addresses (linked via ZOWNER)
//! - ZABCDPHONENUMBER: Phone numbers (linked via ZOWNER)
//! - ZABCDPOSTALADDRESS: Physical addresses
//! - ZABCDNOTE: Contact notes
//!
//! CHANGELOG:
//! - 01/15/2026 - Initial implementation (Claude)

pub mod connection;
pub mod queries;
