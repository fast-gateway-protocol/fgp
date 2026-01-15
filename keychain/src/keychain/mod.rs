//! macOS Keychain access via Security framework.
//!
//! CHANGELOG:
//! - 01/15/2026 - Initial implementation (Claude)

mod queries;

pub use queries::{KeychainStore, PasswordInfo};
