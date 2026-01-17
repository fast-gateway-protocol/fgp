//! API response types for the FGP Skill Registry.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::RegistryStats;
use crate::models::{Category, Skill, SkillSummary};

/// Standard API response wrapper
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ApiError>,
    pub meta: ApiMeta,
}

impl<T> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            meta: ApiMeta::default(),
        }
    }
}

impl ApiResponse<()> {
    pub fn error(code: &str, message: &str) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(ApiError {
                code: code.to_string(),
                message: message.to_string(),
            }),
            meta: ApiMeta::default(),
        }
    }
}

/// API error details
#[derive(Debug, Serialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
}

/// API response metadata
#[derive(Debug, Serialize)]
pub struct ApiMeta {
    pub version: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

impl Default for ApiMeta {
    fn default() -> Self {
        Self {
            version: "1.0.0",
            request_id: None,
        }
    }
}

/// Paginated response
#[derive(Debug, Serialize)]
pub struct PaginatedData<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: u32,
    pub limit: u32,
    pub total_pages: u32,
}

impl<T> From<crate::models::PaginatedResponse<T>> for PaginatedData<T> {
    fn from(pr: crate::models::PaginatedResponse<T>) -> Self {
        Self {
            items: pr.items,
            total: pr.total,
            page: pr.page,
            limit: pr.limit,
            total_pages: pr.total_pages,
        }
    }
}

/// Skill response for API (flattened and simplified)
#[derive(Debug, Serialize)]
pub struct SkillResponse {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub namespace: Option<String>,
    pub full_name: Option<String>,
    pub description: Option<String>,
    pub version: String,
    pub tier: String,
    pub tier_reason: Option<String>,
    pub author: Option<String>,
    pub author_url: Option<String>,
    pub license: Option<String>,
    pub homepage: Option<String>,
    pub keywords: Option<Vec<String>>,
    pub github_stars: i32,
    pub github_forks: Option<i32>,
    pub github_url: Option<String>,
    pub downloads: i32,
    pub featured: bool,
    pub deprecated: bool,
    pub agents: Vec<String>,
    pub fgp_speedup: Option<f64>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<Skill> for SkillResponse {
    fn from(s: Skill) -> Self {
        Self {
            id: s.id,
            slug: s.slug,
            name: s.name,
            namespace: s.namespace,
            full_name: s.full_name,
            description: s.description,
            version: s.version,
            tier: s.tier.as_str().to_string(),
            tier_reason: s.tier_reason,
            author: s.author,
            author_url: s.author_url,
            license: s.license,
            homepage: s.homepage,
            keywords: s.keywords,
            github_stars: s.github_stars,
            github_forks: s.github_forks,
            github_url: s.source_url,
            downloads: s.downloads,
            featured: s.featured,
            deprecated: s.deprecated,
            agents: s.agents.map(|a| a.iter().map(|t| t.display_name().to_string()).collect()).unwrap_or_default(),
            fgp_speedup: s.fgp_speedup,
            created_at: s.created_at,
            updated_at: s.updated_at,
        }
    }
}

/// Skill summary for search results
#[derive(Debug, Serialize)]
pub struct SkillSummaryResponse {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub namespace: Option<String>,
    pub description: Option<String>,
    pub version: String,
    pub tier: String,
    pub github_stars: i32,
    pub downloads: i32,
    pub fgp_speedup: Option<f64>,
    pub featured: bool,
}

impl From<SkillSummary> for SkillSummaryResponse {
    fn from(s: SkillSummary) -> Self {
        Self {
            id: s.id,
            slug: s.slug,
            name: s.name,
            namespace: s.namespace,
            description: s.description,
            version: s.version,
            tier: s.tier.as_str().to_string(),
            github_stars: s.github_stars,
            downloads: s.downloads,
            fgp_speedup: s.fgp_speedup,
            featured: s.featured,
        }
    }
}

/// Category response
#[derive(Debug, Serialize)]
pub struct CategoryResponse {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub skill_count: i32,
}

impl From<Category> for CategoryResponse {
    fn from(c: Category) -> Self {
        Self {
            id: c.id,
            name: c.name,
            slug: c.slug,
            description: c.description,
            icon: c.icon,
            skill_count: c.skill_count,
        }
    }
}

/// Registry stats response
#[derive(Debug, Serialize)]
pub struct StatsResponse {
    pub total_skills: i64,
    pub verified_skills: i64,
    pub trusted_skills: i64,
    pub community_skills: i64,
    pub fgp_accelerated: i64,
    pub total_downloads: i64,
    pub total_categories: i64,
    pub trusted_orgs: i64,
}

impl From<RegistryStats> for StatsResponse {
    fn from(s: RegistryStats) -> Self {
        Self {
            total_skills: s.total_skills,
            verified_skills: s.verified_skills,
            trusted_skills: s.trusted_skills,
            community_skills: s.community_skills,
            fgp_accelerated: s.fgp_accelerated,
            total_downloads: s.total_downloads,
            total_categories: s.total_categories,
            trusted_orgs: s.trusted_orgs,
        }
    }
}

/// Install response with SKILL.md content
#[derive(Debug, Serialize)]
pub struct InstallResponse {
    pub slug: String,
    pub name: String,
    pub version: String,
    pub tier: String,
    pub skill_md: String,
    pub skill_md_hash: String,
}

impl From<Skill> for InstallResponse {
    fn from(s: Skill) -> Self {
        Self {
            slug: s.slug,
            name: s.name,
            version: s.version,
            tier: s.tier.as_str().to_string(),
            skill_md: s.skill_md,
            skill_md_hash: s.skill_md_hash,
        }
    }
}

/// Search query parameters
#[derive(Debug, Deserialize, Default)]
pub struct SearchQuery {
    pub q: Option<String>,
    pub tier: Option<String>,
    pub category: Option<String>,
    pub fgp_only: Option<bool>,
    pub sort: Option<String>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::RegistryStats;
    use crate::models::{
        AgentType, Category, PaginatedResponse, QualityTier, Skill, SkillSummary, SourceType,
    };
    use chrono::Utc;

    #[test]
    fn api_response_ok_and_error() {
        let ok = ApiResponse::ok(123);
        assert!(ok.success);
        assert_eq!(ok.data, Some(123));
        assert!(ok.error.is_none());
        assert_eq!(ok.meta.version, "1.0.0");

        let err = ApiResponse::<()>::error("BAD_REQUEST", "nope");
        assert!(!err.success);
        assert!(err.data.is_none());
        let error = err.error.expect("error");
        assert_eq!(error.code, "BAD_REQUEST");
        assert_eq!(error.message, "nope");
    }

    #[test]
    fn paginated_data_from_response() {
        let response = PaginatedResponse::new(vec![1, 2], 12, 2, 5);
        let data: PaginatedData<i32> = response.into();
        assert_eq!(data.items, vec![1, 2]);
        assert_eq!(data.total, 12);
        assert_eq!(data.page, 2);
        assert_eq!(data.limit, 5);
        assert_eq!(data.total_pages, 3);
    }

    #[test]
    fn summary_and_category_responses_map_fields() {
        let summary = SkillSummary {
            id: Uuid::new_v4(),
            slug: "demo".to_string(),
            name: "Demo".to_string(),
            namespace: Some("acme".to_string()),
            description: Some("desc".to_string()),
            version: "1.0.0".to_string(),
            tier: QualityTier::Community,
            github_stars: 5,
            downloads: 10,
            fgp_speedup: Some(1.2),
            featured: false,
            updated_at: Utc::now(),
        };
        let response = SkillSummaryResponse::from(summary);
        assert_eq!(response.tier, "community");
        assert_eq!(response.downloads, 10);
        assert_eq!(response.github_stars, 5);

        let category = Category {
            id: Uuid::new_v4(),
            name: "Tools".to_string(),
            slug: "tools".to_string(),
            description: Some("desc".to_string()),
            icon: Some("tool".to_string()),
            skill_count: 3,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let cat = CategoryResponse::from(category);
        assert_eq!(cat.slug, "tools");
        assert_eq!(cat.skill_count, 3);
    }

    #[test]
    fn skill_and_install_responses_map_fields() {
        let now = Utc::now();
        let skill = Skill {
            id: Uuid::new_v4(),
            name: "Demo".to_string(),
            slug: "demo".to_string(),
            namespace: Some("acme".to_string()),
            full_name: Some("acme/demo".to_string()),
            description: Some("desc".to_string()),
            long_description: None,
            author: Some("Ada".to_string()),
            author_url: Some("https://example.com".to_string()),
            license: Some("MIT".to_string()),
            homepage: Some("https://example.com/home".to_string()),
            keywords: Some(vec!["tool".to_string()]),
            version: "1.0.0".to_string(),
            source: SourceType::Github,
            source_url: Some("https://github.com/acme/demo".to_string()),
            source_repo: Some("acme/demo".to_string()),
            source_path: None,
            source_branch: Some("main".to_string()),
            source_sha: Some("abc123".to_string()),
            skill_md: "# Demo".to_string(),
            skill_md_hash: "hash".to_string(),
            parsed_frontmatter: None,
            tier: QualityTier::Trusted,
            tier_reason: Some("stars".to_string()),
            verified_at: None,
            verified_by: None,
            agents: Some(vec![AgentType::Codex, AgentType::ClaudeCode]),
            min_agent_versions: None,
            agent_notes: None,
            fgp_daemon_id: None,
            fgp_methods: Some(vec!["demo".to_string()]),
            fgp_speedup: Some(2.5),
            fgp_required: false,
            github_stars: 42,
            github_forks: Some(3),
            github_watchers: None,
            github_open_issues: None,
            github_last_push: None,
            github_created_at: None,
            downloads: 7,
            downloads_week: None,
            downloads_month: None,
            rating_avg: None,
            rating_count: None,
            featured: true,
            deprecated: false,
            deprecated_reason: None,
            deprecated_replacement: None,
            hidden: false,
            security_scanned: false,
            security_scan_at: None,
            security_warnings: None,
            created_at: now,
            updated_at: now,
            synced_at: None,
        };

        let response = SkillResponse::from(skill.clone());
        assert_eq!(response.tier, "trusted");
        assert!(response.agents.contains(&"Codex CLI".to_string()));
        assert!(response.agents.contains(&"Claude Code".to_string()));
        assert_eq!(response.github_stars, 42);

        let install = InstallResponse::from(skill);
        assert_eq!(install.slug, "demo");
        assert_eq!(install.skill_md_hash, "hash");
    }

    #[test]
    fn stats_response_maps_fields() {
        let stats = RegistryStats {
            total_skills: 10,
            verified_skills: 2,
            trusted_skills: 3,
            community_skills: 4,
            fgp_accelerated: 5,
            total_downloads: 6,
            total_categories: 7,
            trusted_orgs: 8,
        };
        let response = StatsResponse::from(stats);
        assert_eq!(response.total_skills, 10);
        assert_eq!(response.trusted_orgs, 8);
    }
}
