//! FGP Skill Registry
//!
//! The canonical skill registry and marketplace for AI coding agents,
//! with FGP acceleration at its core.
//!
//! # Overview
//!
//! This crate provides:
//! - Database models and queries for the skill registry
//! - Sync engine for importing skills from GitHub/SkillsMP
//! - Security scanning for SKILL.md files
//! - Installation tracking and analytics
//!
//! # Quality Tiers
//!
//! Skills are organized into quality tiers:
//! - **Verified** (Tier 0): Manually reviewed, official vendor skills
//! - **Trusted** (Tier 1): 100+ stars, trusted orgs, or has marketplace.json
//! - **Community** (Tier 2): 10+ stars, basic quality threshold
//! - **Unverified** (Tier 3): Everything else, blocked by default
//!
//! # Example
//!
//! ```rust,ignore
//! use fgp_registry::{Database, SkillFilter, QualityTier};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let db = Database::connect_env().await?;
//!
//!     // Search for trusted skills
//!     let filter = SkillFilter::new()
//!         .with_query("browser automation")
//!         .trusted_only();
//!
//!     let results = db.search_skills(&filter).await?;
//!     for skill in results.items {
//!         println!("{} ({:?}) - {} stars",
//!             skill.slug, skill.tier, skill.github_stars);
//!     }
//!
//!     Ok(())
//! }
//! ```

pub mod models;
pub mod db;
pub mod sync;
pub mod security;
pub mod hash;
pub mod skill_import;

#[cfg(feature = "api")]
pub mod api;

// Re-exports
pub use models::{
    AgentType, Category, FgpDaemon, Installation, InstalledSkill, InstallManifest,
    NewSkill, PaginatedResponse, QualityTier, SecurityScanResult, SecuritySeverity,
    SecurityWarning, Skill, SkillDetail, SkillFilter, SkillSearchResult, SkillSort,
    SkillStatsDaily, SkillSummary, SkillVersion, SourceType, TrustedOrg,
};
pub use db::Database;
pub use sync::{SkillsMpClient, GitHubClient, SyncEngine};
pub use security::SecurityScanner;
pub use hash::compute_skill_hash;
