//! Data models for travel search.
//!
//! # CHANGELOG (recent first, max 5 entries)
//! 01/14/2026 - Initial implementation (Claude)

pub mod flight;
pub mod hotel;
pub mod location;

pub use flight::*;
pub use hotel::*;
pub use location::*;
