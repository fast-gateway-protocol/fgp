//! FGP Travel Daemon
//!
//! Fast flight and hotel search via Kiwi/Skypicker GraphQL and Xotelo REST APIs.
//!
//! # CHANGELOG (recent first, max 5 entries)
//! 01/15/2026 - Added local location database for instant lookups (Claude)
//! 01/14/2026 - Initial implementation (Claude)

#![allow(dead_code)]

pub mod api;
pub mod cache;
pub mod error;
pub mod locations;
pub mod models;
pub mod service;

pub use service::TravelService;
