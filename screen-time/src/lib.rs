//! FGP Screen Time Daemon - Fast access to macOS app usage data.
//!
//! This daemon provides fast access to Screen Time data via SQLite queries
//! to the knowledgeC.db database.
//!
//! ## Requirements
//!
//! - Full Disk Access permission must be granted in System Settings
//!
//! CHANGELOG:
//! - 01/15/2026 - Initial implementation (Claude)

pub mod daemon;
pub mod screen_time;

pub use daemon::ScreenTimeService;
