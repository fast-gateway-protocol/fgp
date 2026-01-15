//! FGP Keychain Daemon - Fast access to macOS Keychain.
//!
//! This daemon provides fast access to macOS Keychain
//! using the native Security framework, eliminating subprocess overhead.
//!
//! ## Security Notes
//!
//! - Binary must be code-signed: `codesign -s - ./fgp-keychain-daemon`
//! - First access to a password may trigger macOS approval dialog
//! - Passwords are never logged
//!
//! CHANGELOG:
//! - 01/15/2026 - Initial implementation (Claude)

pub mod daemon;
pub mod keychain;

pub use daemon::KeychainService;
