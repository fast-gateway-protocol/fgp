//! FGP Apple Reminders Daemon - Fast access to Apple Reminders via EventKit.
//!
//! This daemon provides fast access to Apple Reminders
//! using the native EventKit framework, eliminating the ~2.3s MCP cold-start overhead.
//!
//! CHANGELOG:
//! - 01/15/2026 - Initial implementation (Claude)

pub mod reminder;
pub mod daemon;

pub use daemon::RemindersService;
