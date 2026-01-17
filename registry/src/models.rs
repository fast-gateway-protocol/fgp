//! Data models for the FGP Skill Registry
//!
//! These types map to the PostgreSQL schema defined in migrations/001_initial_schema.sql

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ============================================================================
// Enums
// ============================================================================

/// Quality tier levels (higher = more trusted)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "quality_tier", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum QualityTier {
    Unverified,
    Community,
    Trusted,
    Verified,
}

impl QualityTier {
    /// Returns the string representation (lowercase)
    pub fn as_str(&self) -> &'static str {
        match self {
            QualityTier::Unverified => "unverified",
            QualityTier::Community => "community",
            QualityTier::Trusted => "trusted",
            QualityTier::Verified => "verified",
        }
    }

    /// Returns the numeric level (0-3) for comparison
    pub fn level(&self) -> u8 {
        match self {
            QualityTier::Unverified => 0,
            QualityTier::Community => 1,
            QualityTier::Trusted => 2,
            QualityTier::Verified => 3,
        }
    }

    /// Whether this tier allows installation by default
    pub fn installable_by_default(&self) -> bool {
        matches!(self, QualityTier::Community | QualityTier::Trusted | QualityTier::Verified)
    }

    /// Whether this tier requires confirmation before install
    pub fn requires_confirmation(&self) -> bool {
        matches!(self, QualityTier::Community | QualityTier::Trusted)
    }

    /// Calculate tier from GitHub stars and org
    pub fn from_metrics(stars: i32, is_trusted_org: bool, has_marketplace_json: bool) -> Self {
        if is_trusted_org || stars >= 100 || has_marketplace_json {
            QualityTier::Trusted
        } else if stars >= 10 {
            QualityTier::Community
        } else {
            QualityTier::Unverified
        }
    }
}

/// Skill source types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "source_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    Github,
    Skillsmp,
    Direct,
    FgpNative,
}

/// Supported agent targets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "agent_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum AgentType {
    ClaudeCode,
    Codex,
    Gemini,
    Cursor,
    Other,
}

impl AgentType {
    /// Returns the skill directory path for this agent
    pub fn skill_dir(&self) -> &'static str {
        match self {
            AgentType::ClaudeCode => "~/.claude/skills",
            AgentType::Codex => "~/.codex/skills",
            AgentType::Gemini => "~/.gemini/skills",
            AgentType::Cursor => "~/.cursor/skills",
            AgentType::Other => "~/.agent-skills",
        }
    }

    /// Returns the display name
    pub fn display_name(&self) -> &'static str {
        match self {
            AgentType::ClaudeCode => "Claude Code",
            AgentType::Codex => "Codex CLI",
            AgentType::Gemini => "Gemini CLI",
            AgentType::Cursor => "Cursor",
            AgentType::Other => "Other",
        }
    }
}

// ============================================================================
// Core Models
// ============================================================================

/// A skill in the registry
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Skill {
    pub id: Uuid,

    // Identity
    pub name: String,
    pub slug: String,
    pub namespace: Option<String>,
    #[sqlx(skip)]
    pub full_name: Option<String>,

    // Metadata
    pub description: Option<String>,
    pub long_description: Option<String>,
    pub author: Option<String>,
    pub author_url: Option<String>,
    pub license: Option<String>,
    pub homepage: Option<String>,
    pub keywords: Option<Vec<String>>,

    // Versioning
    pub version: String,

    // Source tracking
    pub source: SourceType,
    pub source_url: Option<String>,
    pub source_repo: Option<String>,
    pub source_path: Option<String>,
    pub source_branch: Option<String>,
    pub source_sha: Option<String>,

    // Content
    pub skill_md: String,
    pub skill_md_hash: String,
    pub parsed_frontmatter: Option<serde_json::Value>,

    // Quality & Trust
    pub tier: QualityTier,
    pub tier_reason: Option<String>,
    pub verified_at: Option<DateTime<Utc>>,
    pub verified_by: Option<String>,

    // Agent Compatibility
    pub agents: Option<Vec<AgentType>>,
    pub min_agent_versions: Option<serde_json::Value>,
    pub agent_notes: Option<serde_json::Value>,

    // FGP Integration
    pub fgp_daemon_id: Option<Uuid>,
    pub fgp_methods: Option<Vec<String>>,
    pub fgp_speedup: Option<f64>,
    pub fgp_required: bool,

    // GitHub Metrics
    pub github_stars: i32,
    pub github_forks: Option<i32>,
    pub github_watchers: Option<i32>,
    pub github_open_issues: Option<i32>,
    pub github_last_push: Option<DateTime<Utc>>,
    pub github_created_at: Option<DateTime<Utc>>,

    // Registry Metrics
    pub downloads: i32,
    pub downloads_week: Option<i32>,
    pub downloads_month: Option<i32>,
    pub rating_avg: Option<f64>,
    pub rating_count: Option<i32>,

    // Flags
    pub featured: bool,
    pub deprecated: bool,
    pub deprecated_reason: Option<String>,
    pub deprecated_replacement: Option<String>,
    pub hidden: bool,

    // Security
    pub security_scanned: bool,
    pub security_scan_at: Option<DateTime<Utc>>,
    pub security_warnings: Option<serde_json::Value>,

    // Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub synced_at: Option<DateTime<Utc>>,
}

/// Skill data for creating a new skill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewSkill {
    pub name: String,
    pub slug: String,
    pub namespace: Option<String>,
    pub description: Option<String>,
    pub long_description: Option<String>,
    pub author: Option<String>,
    pub author_url: Option<String>,
    pub license: Option<String>,
    pub homepage: Option<String>,
    pub keywords: Option<Vec<String>>,
    pub version: String,
    pub source: SourceType,
    pub source_url: Option<String>,
    pub source_repo: Option<String>,
    pub source_path: Option<String>,
    pub source_branch: Option<String>,
    pub source_sha: Option<String>,
    pub skill_md: String,
    pub skill_md_hash: String,
    pub parsed_frontmatter: Option<serde_json::Value>,
    pub tier: QualityTier,
    pub tier_reason: Option<String>,
    pub agents: Vec<AgentType>,
    pub github_stars: i32,
    pub github_forks: Option<i32>,
    pub github_last_push: Option<DateTime<Utc>>,
}

/// Compact skill info for list/search responses
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SkillSummary {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub namespace: Option<String>,
    pub description: Option<String>,
    pub version: String,
    pub tier: QualityTier,
    pub github_stars: i32,
    pub downloads: i32,
    pub fgp_speedup: Option<f64>,
    pub featured: bool,
    pub updated_at: DateTime<Utc>,
}

/// A specific version of a skill
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SkillVersion {
    pub id: Uuid,
    pub skill_id: Uuid,
    pub version: String,
    pub skill_md: String,
    pub skill_md_hash: String,
    pub source_sha: Option<String>,
    pub changelog: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// A category for organizing skills
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Category {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub skill_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A trusted organization (auto Tier 1)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TrustedOrg {
    pub id: Uuid,
    pub name: String,
    pub display_name: Option<String>,
    pub url: Option<String>,
    pub tier_override: QualityTier,
    pub created_at: DateTime<Utc>,
}

/// An FGP daemon definition
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FgpDaemon {
    pub id: Uuid,
    pub name: String,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub socket_path: String,
    pub methods: Vec<String>,
    pub version: Option<String>,
    pub avg_speedup: Option<f64>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// An installation event (for analytics)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Installation {
    pub id: Uuid,
    pub skill_id: Uuid,
    pub skill_version: String,
    pub agent: AgentType,
    pub machine_hash: Option<String>,
    pub fgp_version: Option<String>,
    pub os: Option<String>,
    pub installed_at: DateTime<Utc>,
}

/// Daily aggregated stats
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SkillStatsDaily {
    pub id: Uuid,
    pub skill_id: Uuid,
    pub date: NaiveDate,
    pub downloads: i32,
    pub unique_machines: i32,
}

// ============================================================================
// Query/Filter Types
// ============================================================================

/// Sort options for skill queries
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillSort {
    #[default]
    Stars,
    Downloads,
    Recent,
    Name,
    Rating,
}

impl SkillSort {
    pub fn to_sql(&self) -> &'static str {
        match self {
            SkillSort::Stars => "github_stars DESC",
            SkillSort::Downloads => "downloads DESC",
            SkillSort::Recent => "updated_at DESC",
            SkillSort::Name => "name ASC",
            SkillSort::Rating => "rating_avg DESC NULLS LAST",
        }
    }
}

/// Filter options for skill queries
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SkillFilter {
    /// Search query (full-text search)
    pub query: Option<String>,

    /// Filter by category slug
    pub category: Option<String>,

    /// Filter by minimum tier
    pub min_tier: Option<QualityTier>,

    /// Filter by maximum tier (for excluding unverified)
    pub max_tier: Option<QualityTier>,

    /// Filter by agent compatibility
    pub agent: Option<AgentType>,

    /// Only show FGP-accelerated skills
    pub fgp_only: bool,

    /// Only show featured skills
    pub featured_only: bool,

    /// Include deprecated skills
    pub include_deprecated: bool,

    /// Namespace/org filter
    pub namespace: Option<String>,

    /// Minimum GitHub stars
    pub min_stars: Option<i32>,

    /// Sort order
    pub sort: SkillSort,

    /// Pagination
    pub page: u32,
    pub limit: u32,
}

impl SkillFilter {
    pub fn new() -> Self {
        Self {
            page: 1,
            limit: 20,
            ..Default::default()
        }
    }

    pub fn with_query(mut self, query: impl Into<String>) -> Self {
        self.query = Some(query.into());
        self
    }

    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }

    pub fn installable_only(mut self) -> Self {
        self.min_tier = Some(QualityTier::Community);
        self
    }

    pub fn trusted_only(mut self) -> Self {
        self.min_tier = Some(QualityTier::Trusted);
        self
    }

    pub fn offset(&self) -> u32 {
        (self.page.saturating_sub(1)) * self.limit
    }
}

/// Paginated response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: u32,
    pub limit: u32,
    pub total_pages: u32,
}

impl<T> PaginatedResponse<T> {
    pub fn new(items: Vec<T>, total: i64, page: u32, limit: u32) -> Self {
        let total_pages = ((total as f64) / (limit as f64)).ceil() as u32;
        Self {
            items,
            total,
            page,
            limit,
            total_pages,
        }
    }
}

// ============================================================================
// Security Types
// ============================================================================

/// Security warning found during scan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityWarning {
    pub severity: SecuritySeverity,
    pub category: String,
    pub message: String,
    pub line: Option<u32>,
    pub snippet: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SecuritySeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

/// Result of a security scan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityScanResult {
    pub scanned_at: DateTime<Utc>,
    pub passed: bool,
    pub warnings: Vec<SecurityWarning>,
    pub blocked_patterns: Vec<String>,
}

// ============================================================================
// Installation Types
// ============================================================================

/// Result of checking installed skills
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledSkill {
    pub slug: String,
    pub version: String,
    pub tier: QualityTier,
    pub agent: AgentType,
    pub installed_at: DateTime<Utc>,
    pub path: String,
    pub has_update: bool,
    pub latest_version: Option<String>,
}

/// Local manifest of installed skills
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallManifest {
    pub version: u32,
    pub skills: Vec<InstalledSkill>,
    pub updated_at: DateTime<Utc>,
}

impl Default for InstallManifest {
    fn default() -> Self {
        Self {
            version: 1,
            skills: Vec::new(),
            updated_at: Utc::now(),
        }
    }
}

// ============================================================================
// API Response Types
// ============================================================================

/// Skill detail response with full metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDetail {
    #[serde(flatten)]
    pub skill: Skill,
    pub categories: Vec<String>,
    pub fgp_daemon: Option<FgpDaemon>,
    pub recent_versions: Vec<SkillVersion>,
    pub dependencies: Vec<SkillSummary>,
}

/// Search result with highlight info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillSearchResult {
    #[serde(flatten)]
    pub skill: SkillSummary,
    pub categories: Vec<String>,
    pub highlight: Option<String>,
    pub score: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quality_tier_from_metrics() {
        // Unverified: < 10 stars
        assert_eq!(
            QualityTier::from_metrics(5, false, false),
            QualityTier::Unverified
        );

        // Community: 10-99 stars
        assert_eq!(
            QualityTier::from_metrics(50, false, false),
            QualityTier::Community
        );

        // Trusted: 100+ stars
        assert_eq!(
            QualityTier::from_metrics(100, false, false),
            QualityTier::Trusted
        );

        // Trusted: from trusted org
        assert_eq!(
            QualityTier::from_metrics(5, true, false),
            QualityTier::Trusted
        );

        // Trusted: has marketplace.json
        assert_eq!(
            QualityTier::from_metrics(5, false, true),
            QualityTier::Trusted
        );
    }

    #[test]
    fn test_skill_filter_offset() {
        let filter = SkillFilter {
            page: 1,
            limit: 20,
            ..Default::default()
        };
        assert_eq!(filter.offset(), 0);

        let filter = SkillFilter {
            page: 3,
            limit: 20,
            ..Default::default()
        };
        assert_eq!(filter.offset(), 40);
    }

    #[test]
    fn test_paginated_response() {
        let response: PaginatedResponse<i32> = PaginatedResponse::new(vec![1, 2, 3], 100, 1, 20);
        assert_eq!(response.total_pages, 5);
    }
}
