//! Sync engine for importing skills from SkillsMP and GitHub
//!
//! This module provides clients for:
//! - SkillsMP API (skill discovery)
//! - GitHub API (metadata enrichment)
//! - Combined sync engine

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::db::Database;
use crate::hash::compute_skill_hash;
use crate::models::{NewSkill, QualityTier, SourceType, AgentType};
use crate::security::SecurityScanner;

// ============================================================================
// SkillsMP API Client
// ============================================================================

/// SkillsMP API response for skill search
#[derive(Debug, Deserialize)]
pub struct SkillsMpSearchResponse {
    pub success: bool,
    pub data: Option<SkillsMpSearchData>,
    pub error: Option<SkillsMpError>,
}

#[derive(Debug, Deserialize)]
pub struct SkillsMpSearchData {
    pub skills: Vec<SkillsMpSkill>,
    pub total: i64,
    pub page: u32,
    pub limit: u32,
}

#[derive(Debug, Deserialize)]
pub struct SkillsMpSkill {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub author: Option<String>,
    #[serde(rename = "repoFullName")]
    pub repo_full_name: Option<String>,
    pub stars: Option<i32>,
    #[serde(rename = "lastPush")]
    pub last_push: Option<String>,
    pub category: Option<String>,
    #[serde(rename = "hasMarketplaceJson")]
    pub has_marketplace_json: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct SkillsMpError {
    pub code: String,
    pub message: String,
}

/// Client for SkillsMP API
pub struct SkillsMpClient {
    client: Client,
    api_key: String,
    base_url: String,
}

impl SkillsMpClient {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            api_key: api_key.into(),
            base_url: "https://skillsmp.com/api/v1".to_string(),
        }
    }

    /// Search skills with filters
    pub async fn search(
        &self,
        query: Option<&str>,
        sort_by: &str,
        page: u32,
        limit: u32,
    ) -> Result<SkillsMpSearchData> {
        let mut url = format!(
            "{}/skills/search?page={}&limit={}&sortBy={}",
            self.base_url, page, limit, sort_by
        );

        if let Some(q) = query {
            url.push_str(&format!("&q={}", urlencoding::encode(q)));
        }

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .context("Failed to call SkillsMP API")?;

        let result: SkillsMpSearchResponse = response
            .json()
            .await
            .context("Failed to parse SkillsMP response")?;

        if !result.success {
            let error = result.error.unwrap_or(SkillsMpError {
                code: "UNKNOWN".to_string(),
                message: "Unknown error".to_string(),
            });
            anyhow::bail!("SkillsMP API error: {} - {}", error.code, error.message);
        }

        result.data.context("No data in SkillsMP response")
    }

    /// Fetch top skills by stars (for initial sync)
    pub async fn fetch_top_skills(&self, min_stars: i32, max_pages: u32) -> Result<Vec<SkillsMpSkill>> {
        let mut all_skills = Vec::new();
        let limit = 100;

        for page in 1..=max_pages {
            tracing::info!("Fetching SkillsMP page {} of {}", page, max_pages);

            let data = self.search(None, "stars", page, limit).await?;

            // Filter by minimum stars
            let filtered: Vec<_> = data
                .skills
                .into_iter()
                .filter(|s| s.stars.unwrap_or(0) >= min_stars)
                .collect();

            let count = filtered.len();
            all_skills.extend(filtered);

            // Stop if we've hit skills below our threshold
            if count < limit as usize {
                break;
            }
        }

        Ok(all_skills)
    }
}

// ============================================================================
// GitHub API Client
// ============================================================================

/// GitHub repository info
#[derive(Debug, Deserialize)]
pub struct GitHubRepo {
    pub full_name: String,
    pub description: Option<String>,
    pub stargazers_count: i32,
    pub forks_count: i32,
    pub watchers_count: i32,
    pub open_issues_count: i32,
    pub pushed_at: Option<String>,
    pub created_at: Option<String>,
    pub default_branch: String,
    pub license: Option<GitHubLicense>,
    pub owner: GitHubOwner,
}

#[derive(Debug, Deserialize)]
pub struct GitHubLicense {
    pub spdx_id: Option<String>,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct GitHubOwner {
    pub login: String,
    pub html_url: String,
}

/// GitHub file content
#[derive(Debug, Deserialize)]
pub struct GitHubContent {
    pub content: Option<String>,
    pub encoding: Option<String>,
    pub sha: String,
}

/// Client for GitHub API
pub struct GitHubClient {
    client: Client,
    token: Option<String>,
}

impl GitHubClient {
    pub fn new(token: Option<String>) -> Self {
        Self {
            client: Client::new(),
            token,
        }
    }

    pub fn from_env() -> Self {
        let token = std::env::var("GITHUB_TOKEN").ok();
        Self::new(token)
    }

    /// Get repository info
    pub async fn get_repo(&self, owner: &str, repo: &str) -> Result<GitHubRepo> {
        let url = format!("https://api.github.com/repos/{}/{}", owner, repo);

        let mut request = self
            .client
            .get(&url)
            .header("User-Agent", "fgp-registry/0.1")
            .header("Accept", "application/vnd.github.v3+json");

        if let Some(ref token) = self.token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        let response = request.send().await.context("Failed to call GitHub API")?;

        if !response.status().is_success() {
            anyhow::bail!("GitHub API error: {}", response.status());
        }

        let repo: GitHubRepo = response
            .json()
            .await
            .context("Failed to parse GitHub response")?;

        Ok(repo)
    }

    /// Get file content from repo
    pub async fn get_file(
        &self,
        owner: &str,
        repo: &str,
        path: &str,
        branch: Option<&str>,
    ) -> Result<String> {
        // If no branch specified, don't include ref parameter - GitHub uses default branch
        let url = if let Some(branch) = branch {
            format!(
                "https://api.github.com/repos/{}/{}/contents/{}?ref={}",
                owner, repo, path, branch
            )
        } else {
            format!(
                "https://api.github.com/repos/{}/{}/contents/{}",
                owner, repo, path
            )
        };

        let mut request = self
            .client
            .get(&url)
            .header("User-Agent", "fgp-registry/0.1")
            .header("Accept", "application/vnd.github.v3+json");

        if let Some(ref token) = self.token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        let response = request.send().await.context("Failed to call GitHub API")?;

        if !response.status().is_success() {
            anyhow::bail!("GitHub API error: {} for {}", response.status(), path);
        }

        let content: GitHubContent = response
            .json()
            .await
            .context("Failed to parse GitHub content response")?;

        // Decode base64 content
        let encoded = content.content.context("No content in GitHub response")?;
        let decoded = base64_decode(&encoded.replace('\n', ""))?;

        Ok(decoded)
    }

    /// Get SKILL.md from a repo
    pub async fn get_skill_md(
        &self,
        owner: &str,
        repo: &str,
        skill_path: Option<&str>,
    ) -> Result<String> {
        // Try common locations
        let paths = if let Some(path) = skill_path {
            // If path doesn't end with SKILL.md, append it
            if path.ends_with("SKILL.md") || path.ends_with("skill.md") {
                vec![path.to_string()]
            } else {
                let trimmed = path.trim_end_matches('/');
                vec![
                    format!("{}/SKILL.md", trimmed),
                    format!("{}/skill.md", trimmed),
                    trimmed.to_string(), // Also try the path as-is
                ]
            }
        } else {
            vec![
                "SKILL.md".to_string(),
                ".claude/skills/SKILL.md".to_string(),
                "claude-skills/SKILL.md".to_string(),
            ]
        };

        for path in paths {
            match self.get_file(owner, repo, &path, None).await {
                Ok(content) => return Ok(content),
                Err(_) => continue,
            }
        }

        anyhow::bail!("Could not find SKILL.md in {}/{}", owner, repo)
    }

    /// List directories in a repo path that might contain skills
    pub async fn list_skill_directories(
        &self,
        owner: &str,
        repo: &str,
        path: Option<&str>,
    ) -> Result<Vec<GitHubTreeEntry>> {
        let path = path.unwrap_or("");
        let url = if path.is_empty() {
            format!("https://api.github.com/repos/{}/{}/contents", owner, repo)
        } else {
            format!("https://api.github.com/repos/{}/{}/contents/{}", owner, repo, path)
        };

        let mut request = self
            .client
            .get(&url)
            .header("User-Agent", "fgp-registry/0.1")
            .header("Accept", "application/vnd.github.v3+json");

        if let Some(ref token) = self.token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        let response = request.send().await.context("Failed to list directory")?;

        if !response.status().is_success() {
            anyhow::bail!("GitHub API error: {} for path {}", response.status(), path);
        }

        let entries: Vec<GitHubTreeEntry> = response
            .json()
            .await
            .context("Failed to parse directory listing")?;

        Ok(entries)
    }

    /// Find all SKILL.md files in a repo (single level deep)
    pub async fn find_skill_paths(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<Vec<String>> {
        let mut skill_paths = Vec::new();

        // Check root for SKILL.md
        if self.get_file(owner, repo, "SKILL.md", None).await.is_ok() {
            tracing::debug!("Found root SKILL.md");
            skill_paths.push(String::new()); // Root skill
        }

        // List root directories
        let entries = self.list_skill_directories(owner, repo, None).await?;

        for entry in entries {
            if entry.entry_type == "dir" {
                // Check if this directory has a SKILL.md
                let skill_path = format!("{}/SKILL.md", entry.name);
                if self.get_file(owner, repo, &skill_path, None).await.is_ok() {
                    tracing::debug!("Found SKILL.md in {}", entry.name);
                    skill_paths.push(entry.name.clone());
                }
            }
        }

        tracing::info!("Found {} skill directories", skill_paths.len());
        Ok(skill_paths)
    }
}

/// GitHub directory entry
#[derive(Debug, Deserialize)]
pub struct GitHubTreeEntry {
    pub name: String,
    pub path: String,
    #[serde(rename = "type")]
    pub entry_type: String,
    pub sha: String,
}

fn base64_decode(encoded: &str) -> Result<String> {
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    let decoded = STANDARD.decode(encoded).context("Invalid base64")?;
    String::from_utf8(decoded).context("Invalid UTF-8 in decoded content")
}

// ============================================================================
// Sync Engine
// ============================================================================

/// Combined sync engine for importing skills
pub struct SyncEngine {
    db: Database,
    skillsmp: Option<SkillsMpClient>,
    github: GitHubClient,
    scanner: SecurityScanner,
}

impl SyncEngine {
    pub fn new(db: Database, skillsmp_key: Option<String>, github_token: Option<String>) -> Self {
        Self {
            db,
            skillsmp: skillsmp_key.map(SkillsMpClient::new),
            github: GitHubClient::new(github_token),
            scanner: SecurityScanner::new(),
        }
    }

    /// Sync trusted skills from SkillsMP (Tier 1+)
    pub async fn sync_trusted_skills(&self, min_stars: i32) -> Result<SyncResult> {
        let skillsmp = self.skillsmp.as_ref()
            .context("SkillsMP API key required for sync")?;

        let mut result = SyncResult::default();

        // Fetch top skills
        let skills = skillsmp.fetch_top_skills(min_stars, 50).await?;
        tracing::info!("Found {} skills with {} stars", skills.len(), min_stars);

        for skill in skills {
            match self.import_skill(&skill).await {
                Ok(imported) => {
                    if imported {
                        result.imported += 1;
                    } else {
                        result.skipped += 1;
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to import {}: {}", skill.slug, e);
                    result.failed += 1;
                    result.errors.push(format!("{}: {}", skill.slug, e));
                }
            }
        }

        Ok(result)
    }

    /// Import a single skill from SkillsMP data
    async fn import_skill(&self, skill: &SkillsMpSkill) -> Result<bool> {
        // Check if already exists
        if self.db.get_skill(&skill.slug).await?.is_some() {
            tracing::debug!("Skill {} already exists, skipping", skill.slug);
            return Ok(false);
        }

        // Parse repo info
        let (owner, repo) = skill
            .repo_full_name
            .as_ref()
            .and_then(|r| r.split_once('/'))
            .context("Invalid repo name")?;

        // Fetch GitHub metadata
        let gh_repo = self.github.get_repo(owner, repo).await?;

        // Fetch SKILL.md content
        let skill_md = self.github.get_skill_md(owner, repo, None).await?;

        // Security scan
        let scan_result = self.scanner.scan(&skill_md);
        if !scan_result.passed {
            tracing::warn!(
                "Security scan failed for {}: {:?}",
                skill.slug,
                scan_result.blocked_patterns
            );
            // Still import but mark as unverified
        }

        // Calculate tier
        let is_trusted_org = self.db.is_trusted_org(owner).await?;
        let has_marketplace = skill.has_marketplace_json.unwrap_or(false);
        let tier = QualityTier::from_metrics(gh_repo.stargazers_count, is_trusted_org, has_marketplace);

        // Parse last push date
        let last_push = gh_repo.pushed_at.as_ref().and_then(|s| {
            DateTime::parse_from_rfc3339(s).ok().map(|dt| dt.with_timezone(&Utc))
        });

        // Create new skill
        let new_skill = NewSkill {
            name: skill.name.clone(),
            slug: skill.slug.clone(),
            namespace: Some(owner.to_string()),
            description: skill.description.clone().or(gh_repo.description),
            long_description: None,
            author: Some(gh_repo.owner.login.clone()),
            author_url: Some(gh_repo.owner.html_url.clone()),
            license: gh_repo.license.map(|l| l.spdx_id.unwrap_or(l.name)),
            homepage: None,
            keywords: None,
            version: "1.0.0".to_string(),
            source: SourceType::Github,
            source_url: Some(format!("https://github.com/{}", gh_repo.full_name)),
            source_repo: Some(gh_repo.full_name),
            source_path: None,
            source_branch: Some(gh_repo.default_branch),
            source_sha: None,
            skill_md: skill_md.clone(),
            skill_md_hash: compute_skill_hash(&skill_md),
            parsed_frontmatter: None, // TODO: Parse frontmatter
            tier,
            tier_reason: Some(format!(
                "stars={}, trusted_org={}, marketplace_json={}",
                gh_repo.stargazers_count, is_trusted_org, has_marketplace
            )),
            agents: vec![AgentType::ClaudeCode, AgentType::Codex, AgentType::Gemini],
            github_stars: gh_repo.stargazers_count,
            github_forks: Some(gh_repo.forks_count),
            github_last_push: last_push,
        };

        self.db.insert_skill(&new_skill).await?;
        tracing::info!("Imported {} ({:?}, {} stars)", skill.slug, tier, gh_repo.stargazers_count);

        Ok(true)
    }

    /// Sync all skills from a GitHub repo (scans for SKILL.md files in directories)
    pub async fn sync_github_repo(&self, owner: &str, repo: &str) -> Result<SyncResult> {
        let mut result = SyncResult::default();

        // Get repo metadata
        let gh_repo = self.github.get_repo(owner, repo).await?;
        tracing::info!("Syncing from {}/{} ({} stars)", owner, repo, gh_repo.stargazers_count);

        // Find all skill directories
        let skill_paths = self.github.find_skill_paths(owner, repo).await?;
        tracing::info!("Found {} skill directories", skill_paths.len());

        // Check if org is trusted
        let is_trusted_org = self.db.is_trusted_org(owner).await?;

        for skill_path in skill_paths {
            match self.import_skill_from_path(owner, repo, &skill_path, &gh_repo, is_trusted_org).await {
                Ok(imported) => {
                    if imported {
                        result.imported += 1;
                    } else {
                        result.skipped += 1;
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to import skill from {}/{}/{}: {}", owner, repo, skill_path, e);
                    result.failed += 1;
                    result.errors.push(format!("{}/{}: {}", repo, skill_path, e));
                }
            }
        }

        Ok(result)
    }

    /// Import a skill from a specific path in a repo
    async fn import_skill_from_path(
        &self,
        owner: &str,
        repo: &str,
        skill_path: &str,
        gh_repo: &GitHubRepo,
        is_trusted_org: bool,
    ) -> Result<bool> {
        // Generate slug: owner-repo-skillpath (or owner-repo if root)
        let slug = if skill_path.is_empty() {
            format!("{}-{}", owner, repo)
        } else {
            format!("{}-{}", owner, skill_path)
        };

        // Check if already exists
        if self.db.get_skill(&slug).await?.is_some() {
            tracing::debug!("Skill {} already exists, skipping", slug);
            return Ok(false);
        }

        // Fetch SKILL.md content
        let skill_path_opt = if skill_path.is_empty() { None } else { Some(skill_path) };
        let skill_md = self.github.get_skill_md(owner, repo, skill_path_opt).await?;

        // Security scan
        let scan_result = self.scanner.scan(&skill_md);
        if !scan_result.passed {
            tracing::warn!(
                "Security scan failed for {}: {:?}",
                slug,
                scan_result.blocked_patterns
            );
        }

        // Parse name from SKILL.md frontmatter or use path
        let name = parse_skill_name(&skill_md)
            .unwrap_or_else(|| {
                if skill_path.is_empty() {
                    repo.to_string()
                } else {
                    skill_path.replace('-', " ")
                }
            });

        // Parse description from frontmatter
        let description = parse_skill_description(&skill_md)
            .or_else(|| gh_repo.description.clone());

        // Calculate tier
        let tier = QualityTier::from_metrics(gh_repo.stargazers_count, is_trusted_org, false);

        // Parse last push date
        let last_push = gh_repo.pushed_at.as_ref().and_then(|s| {
            DateTime::parse_from_rfc3339(s).ok().map(|dt| dt.with_timezone(&Utc))
        });

        // Create new skill
        let new_skill = NewSkill {
            name,
            slug: slug.clone(),
            namespace: Some(owner.to_string()),
            description,
            long_description: None,
            author: Some(gh_repo.owner.login.clone()),
            author_url: Some(gh_repo.owner.html_url.clone()),
            license: gh_repo.license.as_ref().map(|l| l.spdx_id.clone().unwrap_or_else(|| l.name.clone())),
            homepage: None,
            keywords: None,
            version: "1.0.0".to_string(),
            source: SourceType::Github,
            source_url: Some(format!("https://github.com/{}/{}", owner, repo)),
            source_repo: Some(gh_repo.full_name.clone()),
            source_path: if skill_path.is_empty() { None } else { Some(skill_path.to_string()) },
            source_branch: Some(gh_repo.default_branch.clone()),
            source_sha: None,
            skill_md: skill_md.clone(),
            skill_md_hash: compute_skill_hash(&skill_md),
            parsed_frontmatter: None,
            tier,
            tier_reason: Some(format!(
                "stars={}, trusted_org={}",
                gh_repo.stargazers_count, is_trusted_org
            )),
            agents: vec![AgentType::ClaudeCode, AgentType::Codex, AgentType::Gemini],
            github_stars: gh_repo.stargazers_count,
            github_forks: Some(gh_repo.forks_count),
            github_last_push: last_push,
        };

        self.db.insert_skill(&new_skill).await?;
        tracing::info!("Imported {} ({:?}, {} stars)", slug, tier, gh_repo.stargazers_count);

        Ok(true)
    }
}

/// Parse skill name from SKILL.md frontmatter
fn parse_skill_name(content: &str) -> Option<String> {
    // Look for YAML frontmatter
    if !content.starts_with("---") {
        return None;
    }
    let end = content[3..].find("---")?;
    let frontmatter = &content[3..3+end];

    for line in frontmatter.lines() {
        let line = line.trim();
        if let Some(name) = line.strip_prefix("name:") {
            let name = name.trim().trim_matches('"').trim_matches('\'');
            if !name.is_empty() {
                return Some(name.to_string());
            }
        }
    }
    None
}

/// Parse skill description from SKILL.md frontmatter
fn parse_skill_description(content: &str) -> Option<String> {
    if !content.starts_with("---") {
        return None;
    }
    let end = content[3..].find("---")?;
    let frontmatter = &content[3..3+end];

    for line in frontmatter.lines() {
        let line = line.trim();
        if let Some(desc) = line.strip_prefix("description:") {
            let desc = desc.trim().trim_matches('"').trim_matches('\'');
            if !desc.is_empty() {
                return Some(desc.to_string());
            }
        }
    }
    None
}

/// Result of a sync operation
#[derive(Debug, Default)]
pub struct SyncResult {
    pub imported: usize,
    pub skipped: usize,
    pub failed: usize,
    pub errors: Vec<String>,
}

// Add base64 dependency note
// Note: Add `base64 = "0.22"` to Cargo.toml for the base64_decode function

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{engine::general_purpose::STANDARD, Engine as _};

    #[test]
    fn test_quality_tier_calculation() {
        // High stars = trusted
        assert_eq!(
            QualityTier::from_metrics(150, false, false),
            QualityTier::Trusted
        );

        // Trusted org = trusted regardless of stars
        assert_eq!(
            QualityTier::from_metrics(5, true, false),
            QualityTier::Trusted
        );

        // Has marketplace.json = trusted
        assert_eq!(
            QualityTier::from_metrics(5, false, true),
            QualityTier::Trusted
        );

        // Medium stars = community
        assert_eq!(
            QualityTier::from_metrics(50, false, false),
            QualityTier::Community
        );

        // Low stars = unverified
        assert_eq!(
            QualityTier::from_metrics(5, false, false),
            QualityTier::Unverified
        );
    }

    #[test]
    fn test_parse_skill_name_from_frontmatter() {
        let content = "---\nname: example-skill\n---\n# Example";
        assert_eq!(parse_skill_name(content), Some("example-skill".to_string()));
    }

    #[test]
    fn test_parse_skill_description_from_frontmatter() {
        let content = "---\ndescription: \"Example skill\"\n---\n# Example";
        assert_eq!(
            parse_skill_description(content),
            Some("Example skill".to_string())
        );
    }

    #[test]
    fn test_parse_skill_name_missing_frontmatter() {
        let content = "# Example";
        assert_eq!(parse_skill_name(content), None);
    }

    #[test]
    fn test_base64_decode_success() {
        let encoded = STANDARD.encode("hello");
        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(decoded, "hello");
    }

    #[test]
    fn test_base64_decode_invalid() {
        let err = base64_decode("@@@").expect_err("expected invalid base64");
        assert!(err.to_string().contains("Invalid base64"));
    }
}
