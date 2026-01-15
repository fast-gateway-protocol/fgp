//! FGP Apple Calendar Daemon - Fast access to Apple Calendar via EventKit.
//!
//! This daemon provides fast access to Apple Calendar events and calendars
//! using the native EventKit framework, eliminating the ~2.3s MCP cold-start overhead.
//!
//! CHANGELOG:
//! - 01/15/2026 - Initial implementation (Claude)

pub mod calendar;
pub mod daemon;

pub use daemon::CalendarService;
