//! LRU cache with TTL support.
//!
//! # CHANGELOG (recent first, max 5 entries)
//! 01/14/2026 - Initial implementation (Claude)

mod lru;

pub use lru::{CacheStats, TtlCache};
