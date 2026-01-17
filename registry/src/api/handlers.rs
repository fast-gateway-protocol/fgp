//! Request handlers for the FGP Skill Registry API.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;

use crate::db::Database;
use crate::models::{QualityTier, SkillFilter, SkillSort};

use super::responses::*;

/// Application state shared across handlers
pub struct AppState {
    pub db: Database,
    /// Optional admin API key for protected endpoints
    pub admin_api_key: Option<String>,
}

/// GET /api/v1/skills - Search skills
pub async fn search_skills(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchQuery>,
) -> Result<Json<ApiResponse<PaginatedData<SkillSummaryResponse>>>, (StatusCode, Json<ApiResponse<()>>)> {
    let mut filter = SkillFilter::new();

    // Apply query
    if let Some(ref q) = params.q {
        filter = filter.with_query(q);
    }

    // Apply tier filter
    if let Some(ref tier) = params.tier {
        filter.min_tier = match tier.to_lowercase().as_str() {
            "verified" => Some(QualityTier::Verified),
            "trusted" => Some(QualityTier::Trusted),
            "community" => Some(QualityTier::Community),
            "unverified" | "all" => None,
            _ => Some(QualityTier::Community), // Default to community+
        };
    } else {
        // Default to community+ (installable)
        filter.min_tier = Some(QualityTier::Community);
    }

    // Apply category filter
    if let Some(ref category) = params.category {
        filter = filter.with_category(category);
    }

    // Apply FGP filter
    filter.fgp_only = params.fgp_only.unwrap_or(false);

    // Apply sort
    if let Some(ref sort) = params.sort {
        filter.sort = match sort.to_lowercase().as_str() {
            "stars" => SkillSort::Stars,
            "downloads" => SkillSort::Downloads,
            "recent" => SkillSort::Recent,
            "name" => SkillSort::Name,
            _ => SkillSort::Stars,
        };
    }

    // Apply pagination
    filter.page = params.page.unwrap_or(1);
    filter.limit = params.limit.unwrap_or(20).min(100);

    // Execute search
    match state.db.search_skills(&filter).await {
        Ok(results) => {
            let items: Vec<SkillSummaryResponse> = results
                .items
                .into_iter()
                .map(SkillSummaryResponse::from)
                .collect();

            let data = PaginatedData {
                items,
                total: results.total,
                page: results.page,
                limit: results.limit,
                total_pages: results.total_pages,
            };

            Ok(Json(ApiResponse::ok(data)))
        }
        Err(e) => {
            tracing::error!("Search error: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("SEARCH_ERROR", &e.to_string())),
            ))
        }
    }
}

/// GET /api/v1/skills/:slug - Get skill details
pub async fn get_skill(
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
) -> Result<Json<ApiResponse<SkillResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    match state.db.get_skill(&slug).await {
        Ok(Some(skill)) => Ok(Json(ApiResponse::ok(SkillResponse::from(skill)))),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("NOT_FOUND", &format!("Skill '{}' not found", slug))),
        )),
        Err(e) => {
            tracing::error!("Get skill error: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("DATABASE_ERROR", &e.to_string())),
            ))
        }
    }
}

/// GET /api/v1/skills/:slug/install - Get skill content for installation
pub async fn install_skill(
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
) -> Result<Json<ApiResponse<InstallResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    match state.db.get_skill(&slug).await {
        Ok(Some(skill)) => {
            // Increment download counter
            if let Err(e) = state.db.increment_downloads(skill.id).await {
                tracing::warn!("Failed to increment downloads for {}: {}", slug, e);
            }

            Ok(Json(ApiResponse::ok(InstallResponse::from(skill))))
        }
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("NOT_FOUND", &format!("Skill '{}' not found", slug))),
        )),
        Err(e) => {
            tracing::error!("Install skill error: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("DATABASE_ERROR", &e.to_string())),
            ))
        }
    }
}

/// GET /api/v1/categories - List all categories
pub async fn list_categories(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<Vec<CategoryResponse>>>, (StatusCode, Json<ApiResponse<()>>)> {
    match state.db.get_categories().await {
        Ok(categories) => {
            let data: Vec<CategoryResponse> = categories
                .into_iter()
                .map(CategoryResponse::from)
                .collect();
            Ok(Json(ApiResponse::ok(data)))
        }
        Err(e) => {
            tracing::error!("List categories error: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("DATABASE_ERROR", &e.to_string())),
            ))
        }
    }
}

/// GET /api/v1/stats - Get registry statistics
pub async fn get_stats(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<StatsResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    match state.db.get_stats().await {
        Ok(stats) => Ok(Json(ApiResponse::ok(StatsResponse::from(stats)))),
        Err(e) => {
            tracing::error!("Get stats error: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("DATABASE_ERROR", &e.to_string())),
            ))
        }
    }
}

/// GET /health - Health check endpoint
pub async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "fgp-registry-api",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

// ============================================================================
// Publish / Submit Endpoints
// ============================================================================

use serde::Deserialize;

/// Request body for publishing a skill
#[derive(Debug, Deserialize)]
pub struct PublishRequest {
    /// The SKILL.md content
    pub skill_md: String,

    /// Optional slug (auto-generated if not provided)
    pub slug: Option<String>,

    /// Source repository URL (e.g., "https://github.com/owner/repo")
    pub source_url: Option<String>,

    /// Path within the repository (for monorepos)
    pub source_path: Option<String>,
}

/// Response for publish endpoint
#[derive(Debug, serde::Serialize)]
pub struct PublishResponse {
    pub slug: String,
    pub name: String,
    pub version: String,
    pub tier: String,
    pub created: bool,
    pub message: String,
}

/// POST /api/v1/skills - Publish/submit a new skill
pub async fn publish_skill(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(request): Json<PublishRequest>,
) -> Result<Json<ApiResponse<PublishResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Verify API key for publish operations (uses state.admin_api_key)
    if let Some(ref expected_key) = state.admin_api_key {
        let provided_key = headers
            .get("X-API-Key")
            .and_then(|v| v.to_str().ok());

        match provided_key {
            Some(key) if key == expected_key => {
                // Valid key
            }
            _ => {
                return Err((
                    StatusCode::UNAUTHORIZED,
                    Json(ApiResponse::error("UNAUTHORIZED", "Valid X-API-Key header required for publishing")),
                ));
            }
        }
    }

    // Parse the SKILL.md content
    let parsed = crate::skill_import::parse_skill_md(&request.skill_md)
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::error("INVALID_SKILL_MD", &format!("Failed to parse SKILL.md: {}", e))),
            )
        })?;

    // Generate slug if not provided
    let slug = request.slug.unwrap_or_else(|| {
        // Generate from name
        parsed.name.to_lowercase()
            .replace(' ', "-")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-')
            .collect()
    });

    // Check if skill already exists
    let existing = state.db.get_skill(&slug).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error("DATABASE_ERROR", &e.to_string())),
        )
    })?;

    if existing.is_some() {
        return Err((
            StatusCode::CONFLICT,
            Json(ApiResponse::error("SKILL_EXISTS", &format!("Skill '{}' already exists", slug))),
        ));
    }

    // Security scan
    let scanner = crate::security::SecurityScanner::new();
    let scan_result = scanner.scan(&request.skill_md);

    let tier = if !scan_result.passed {
        tracing::warn!("Security scan failed for {}: {:?}", slug, scan_result.blocked_patterns);
        QualityTier::Unverified
    } else {
        // Default to Community tier for published skills
        QualityTier::Community
    };

    // Create the skill
    let new_skill = crate::models::NewSkill {
        name: parsed.name.clone(),
        slug: slug.clone(),
        namespace: None, // Will be set from source URL if provided
        description: parsed.description.clone(),
        long_description: None,
        author: parsed.author.clone(),
        author_url: None,
        license: parsed.license.clone(),
        homepage: None,
        keywords: parsed.keywords.clone(),
        version: parsed.version.clone().unwrap_or_else(|| "1.0.0".to_string()),
        source: crate::models::SourceType::Direct,
        source_url: request.source_url.clone(),
        source_repo: None,
        source_path: request.source_path.clone(),
        source_branch: None,
        source_sha: None,
        skill_md: request.skill_md.clone(),
        skill_md_hash: crate::hash::compute_skill_hash(&request.skill_md),
        parsed_frontmatter: None,
        tier,
        tier_reason: Some(if scan_result.passed {
            "Published via API".to_string()
        } else {
            format!("Security scan warnings: {:?}", scan_result.blocked_patterns)
        }),
        agents: parsed.agents.unwrap_or_else(|| vec![
            crate::models::AgentType::ClaudeCode,
            crate::models::AgentType::Codex,
        ]),
        github_stars: 0,
        github_forks: None,
        github_last_push: None,
    };

    let skill = state.db.insert_skill(&new_skill).await.map_err(|e| {
        tracing::error!("Failed to insert skill: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error("DATABASE_ERROR", &format!("Failed to create skill: {}", e))),
        )
    })?;

    tracing::info!("Published new skill: {} (tier: {:?})", slug, tier);

    Ok(Json(ApiResponse::ok(PublishResponse {
        slug: skill.slug,
        name: skill.name,
        version: skill.version,
        tier: skill.tier.as_str().to_string(),
        created: true,
        message: "Skill published successfully".to_string(),
    })))
}
