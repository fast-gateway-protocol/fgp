//! FGP System Daemon - Cached macOS system information.
//!
//! This daemon provides fast access to system information with intelligent caching.
//! Eliminates the ~2.3s MCP cold-start overhead and caches slow queries like
//! system_profiler (191ms → <1ms cached) and diskutil (1.1s → <1ms cached).
//!
//! CHANGELOG:
//! - 01/15/2026 - Initial implementation (Claude)

pub mod daemon;
pub mod system;

pub use daemon::SystemService;
