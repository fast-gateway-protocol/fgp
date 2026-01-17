//! Database operations for the skill registry

use anyhow::{Context, Result};
use sqlx::postgres::{PgPool, PgPoolOptions};
use uuid::Uuid;

use crate::models::{
    Category, FgpDaemon, NewSkill, PaginatedResponse, QualityTier, Skill, SkillFilter,
    SkillSummary, SkillVersion, TrustedOrg,
};

/// Database connection pool and operations
#[derive(Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    /// Create a new database connection from a connection string
    pub async fn connect(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await
            .context("Failed to connect to database")?;

        Ok(Self { pool })
    }

    /// Create a new database connection from DATABASE_URL env var
    pub async fn connect_env() -> Result<Self> {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL")
            .context("DATABASE_URL must be set")?;
        Self::connect(&database_url).await
    }

    /// Run migrations
    pub async fn migrate(&self) -> Result<()> {
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await
            .context("Failed to run migrations")?;
        Ok(())
    }

    /// Get the underlying pool (for advanced queries)
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    // ========================================================================
    // Skills
    // ========================================================================

    /// Get a skill by slug
    pub async fn get_skill(&self, slug: &str) -> Result<Option<Skill>> {
        let skill = sqlx::query_as::<_, Skill>(
            r#"
            SELECT * FROM skills
            WHERE slug = $1 AND hidden = FALSE
            "#,
        )
        .bind(slug)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch skill")?;

        Ok(skill)
    }

    /// Get a skill by ID
    pub async fn get_skill_by_id(&self, id: Uuid) -> Result<Option<Skill>> {
        let skill = sqlx::query_as::<_, Skill>(
            r#"
            SELECT * FROM skills
            WHERE id = $1 AND hidden = FALSE
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch skill")?;

        Ok(skill)
    }

    /// Search skills with filters
    pub async fn search_skills(
        &self,
        filter: &SkillFilter,
    ) -> Result<PaginatedResponse<SkillSummary>> {
        // For simplicity with dynamic queries, we'll build SQL with inline values
        // (properly escaped) rather than using parameter binding
        let mut conditions = vec!["hidden = FALSE".to_string()];

        // Full-text search - escape single quotes to prevent SQL injection
        if let Some(ref query) = filter.query {
            let escaped = query.replace('\'', "''");
            conditions.push(format!(
                "search_vector @@ plainto_tsquery('english', '{}')",
                escaped
            ));
        }

        // Tier filters
        if let Some(min_tier) = filter.min_tier {
            let tier_values = match min_tier {
                QualityTier::Unverified => "'unverified', 'community', 'trusted', 'verified'",
                QualityTier::Community => "'community', 'trusted', 'verified'",
                QualityTier::Trusted => "'trusted', 'verified'",
                QualityTier::Verified => "'verified'",
            };
            conditions.push(format!("tier::text IN ({tier_values})"));
        }

        // FGP only
        if filter.fgp_only {
            conditions.push("fgp_daemon_id IS NOT NULL".to_string());
        }

        // Featured only
        if filter.featured_only {
            conditions.push("featured = TRUE".to_string());
        }

        // Deprecated filter
        if !filter.include_deprecated {
            conditions.push("deprecated = FALSE".to_string());
        }

        // Namespace filter
        if let Some(ref namespace) = filter.namespace {
            let escaped = namespace.replace('\'', "''");
            conditions.push(format!("namespace = '{}'", escaped));
        }

        // Category filter
        if let Some(ref category) = filter.category {
            let escaped = category.replace('\'', "''");
            conditions.push(format!(
                "id IN (SELECT skill_id FROM skill_categories sc
                        JOIN categories c ON sc.category_id = c.id
                        WHERE c.slug = '{}')",
                escaped
            ));
        }

        // Minimum stars
        if let Some(min_stars) = filter.min_stars {
            conditions.push(format!("github_stars >= {}", min_stars));
        }

        let where_clause = conditions.join(" AND ");
        let order_by = filter.sort.to_sql();
        let limit = filter.limit.min(100);
        let offset = filter.offset();

        // Count total
        let count_sql = format!(
            "SELECT COUNT(*) FROM skills WHERE {where_clause}"
        );

        let total: i64 = sqlx::query_scalar(&count_sql)
            .fetch_one(&self.pool)
            .await
            .context("Failed to count skills")?;

        // Fetch page
        let query_sql = format!(
            r#"
            SELECT
                id, slug, name, namespace, description, version,
                tier, github_stars, downloads, fgp_speedup, featured, updated_at
            FROM skills
            WHERE {where_clause}
            ORDER BY {order_by}
            LIMIT {limit} OFFSET {offset}
            "#
        );

        let skills = sqlx::query_as::<_, SkillSummary>(&query_sql)
            .fetch_all(&self.pool)
            .await
            .context("Failed to search skills")?;

        Ok(PaginatedResponse::new(skills, total, filter.page, limit))
    }

    /// Insert a new skill
    pub async fn insert_skill(&self, skill: &NewSkill) -> Result<Skill> {
        let row = sqlx::query_as::<_, Skill>(
            r#"
            INSERT INTO skills (
                name, slug, namespace, description, long_description,
                author, author_url, license, homepage, keywords,
                version, source, source_url, source_repo, source_path,
                source_branch, source_sha, skill_md, skill_md_hash,
                parsed_frontmatter, tier, tier_reason, agents,
                github_stars, github_forks, github_last_push
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                $11, $12, $13, $14, $15, $16, $17, $18, $19,
                $20, $21, $22, $23, $24, $25, $26
            )
            RETURNING *
            "#,
        )
        .bind(&skill.name)
        .bind(&skill.slug)
        .bind(&skill.namespace)
        .bind(&skill.description)
        .bind(&skill.long_description)
        .bind(&skill.author)
        .bind(&skill.author_url)
        .bind(&skill.license)
        .bind(&skill.homepage)
        .bind(&skill.keywords)
        .bind(&skill.version)
        .bind(&skill.source)
        .bind(&skill.source_url)
        .bind(&skill.source_repo)
        .bind(&skill.source_path)
        .bind(&skill.source_branch)
        .bind(&skill.source_sha)
        .bind(&skill.skill_md)
        .bind(&skill.skill_md_hash)
        .bind(&skill.parsed_frontmatter)
        .bind(&skill.tier)
        .bind(&skill.tier_reason)
        .bind(&skill.agents)
        .bind(skill.github_stars)
        .bind(&skill.github_forks)
        .bind(&skill.github_last_push)
        .fetch_one(&self.pool)
        .await
        .context("Failed to insert skill")?;

        Ok(row)
    }

    /// Update skill metrics from GitHub
    pub async fn update_skill_metrics(
        &self,
        skill_id: Uuid,
        stars: i32,
        forks: Option<i32>,
        last_push: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE skills
            SET github_stars = $2,
                github_forks = COALESCE($3, github_forks),
                github_last_push = COALESCE($4, github_last_push),
                synced_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(skill_id)
        .bind(stars)
        .bind(forks)
        .bind(last_push)
        .execute(&self.pool)
        .await
        .context("Failed to update skill metrics")?;

        Ok(())
    }

    /// Increment download counter
    pub async fn increment_downloads(&self, skill_id: Uuid) -> Result<()> {
        sqlx::query("SELECT increment_downloads($1)")
            .bind(skill_id)
            .execute(&self.pool)
            .await
            .context("Failed to increment downloads")?;

        Ok(())
    }

    // ========================================================================
    // Categories
    // ========================================================================

    /// Get all categories
    pub async fn get_categories(&self) -> Result<Vec<Category>> {
        let categories = sqlx::query_as::<_, Category>(
            "SELECT * FROM categories ORDER BY skill_count DESC",
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch categories")?;

        Ok(categories)
    }

    /// Get category by slug
    pub async fn get_category(&self, slug: &str) -> Result<Option<Category>> {
        let category = sqlx::query_as::<_, Category>(
            "SELECT * FROM categories WHERE slug = $1",
        )
        .bind(slug)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch category")?;

        Ok(category)
    }

    // ========================================================================
    // Trusted Orgs
    // ========================================================================

    /// Check if an org is trusted
    pub async fn is_trusted_org(&self, org_name: &str) -> Result<bool> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM trusted_orgs WHERE name = $1)",
        )
        .bind(org_name)
        .fetch_one(&self.pool)
        .await
        .context("Failed to check trusted org")?;

        Ok(exists)
    }

    /// Get all trusted orgs
    pub async fn get_trusted_orgs(&self) -> Result<Vec<TrustedOrg>> {
        let orgs = sqlx::query_as::<_, TrustedOrg>(
            "SELECT * FROM trusted_orgs ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch trusted orgs")?;

        Ok(orgs)
    }

    // ========================================================================
    // FGP Daemons
    // ========================================================================

    /// Get all FGP daemons
    pub async fn get_fgp_daemons(&self) -> Result<Vec<FgpDaemon>> {
        let daemons = sqlx::query_as::<_, FgpDaemon>(
            "SELECT * FROM fgp_daemons WHERE status = 'active' ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch FGP daemons")?;

        Ok(daemons)
    }

    /// Get FGP daemon by name
    pub async fn get_fgp_daemon(&self, name: &str) -> Result<Option<FgpDaemon>> {
        let daemon = sqlx::query_as::<_, FgpDaemon>(
            "SELECT * FROM fgp_daemons WHERE name = $1",
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch FGP daemon")?;

        Ok(daemon)
    }

    // ========================================================================
    // Skill Versions
    // ========================================================================

    /// Get versions for a skill
    pub async fn get_skill_versions(&self, skill_id: Uuid) -> Result<Vec<SkillVersion>> {
        let versions = sqlx::query_as::<_, SkillVersion>(
            r#"
            SELECT * FROM skill_versions
            WHERE skill_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(skill_id)
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch skill versions")?;

        Ok(versions)
    }

    /// Insert a new skill version
    pub async fn insert_skill_version(
        &self,
        skill_id: Uuid,
        version: &str,
        skill_md: &str,
        skill_md_hash: &str,
        source_sha: Option<&str>,
        changelog: Option<&str>,
    ) -> Result<SkillVersion> {
        let row = sqlx::query_as::<_, SkillVersion>(
            r#"
            INSERT INTO skill_versions (skill_id, version, skill_md, skill_md_hash, source_sha, changelog)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(skill_id)
        .bind(version)
        .bind(skill_md)
        .bind(skill_md_hash)
        .bind(source_sha)
        .bind(changelog)
        .fetch_one(&self.pool)
        .await
        .context("Failed to insert skill version")?;

        Ok(row)
    }

    // ========================================================================
    // Statistics
    // ========================================================================

    /// Get registry statistics
    pub async fn get_stats(&self) -> Result<RegistryStats> {
        let stats = sqlx::query_as::<_, RegistryStats>(
            r#"
            SELECT
                (SELECT COUNT(*) FROM skills WHERE hidden = FALSE) as total_skills,
                (SELECT COUNT(*) FROM skills WHERE tier = 'verified') as verified_skills,
                (SELECT COUNT(*) FROM skills WHERE tier = 'trusted') as trusted_skills,
                (SELECT COUNT(*) FROM skills WHERE tier = 'community') as community_skills,
                (SELECT COUNT(*) FROM skills WHERE fgp_daemon_id IS NOT NULL) as fgp_accelerated,
                (SELECT COALESCE(SUM(downloads), 0) FROM skills) as total_downloads,
                (SELECT COUNT(*) FROM categories) as total_categories,
                (SELECT COUNT(*) FROM trusted_orgs) as trusted_orgs
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to fetch registry stats")?;

        Ok(stats)
    }
}

/// Registry-wide statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct RegistryStats {
    pub total_skills: i64,
    pub verified_skills: i64,
    pub trusted_skills: i64,
    pub community_skills: i64,
    pub fgp_accelerated: i64,
    pub total_downloads: i64,
    pub total_categories: i64,
    pub trusted_orgs: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Integration tests would go here, requiring a test database
    // For now, we just test that types compile correctly

    #[test]
    fn test_skill_filter_builds() {
        let filter = SkillFilter::new()
            .with_query("browser")
            .trusted_only();

        assert_eq!(filter.query, Some("browser".to_string()));
        assert_eq!(filter.min_tier, Some(QualityTier::Trusted));
    }
}
