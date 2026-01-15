//! FGP Travel Daemon
//!
//! Fast flight and hotel search via Kiwi/Skypicker GraphQL and Xotelo REST APIs.
//!
//! # CHANGELOG (recent first, max 5 entries)
//! 01/14/2026 - Initial implementation (Claude)

pub mod api;
pub mod cache;
pub mod error;
pub mod models;
pub mod service;

pub use service::TravelService;
