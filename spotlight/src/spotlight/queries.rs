//! Spotlight query implementations using mdquery-rs.
//!
//! CHANGELOG:
//! - 01/15/2026 - Initial implementation (Claude)

use anyhow::{anyhow, Result};
use chrono::{Duration, Utc};
use mdquery_rs::{MDItem, MDItemKey, MDQuery, MDQueryBuilder, MDQueryCompareOp, MDQueryScope};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A search result item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub path: String,
    pub name: String,
    pub kind: Option<String>,
    pub size: Option<u64>,
    pub modified: Option<String>,
    pub created: Option<String>,
}

impl SearchResult {
    /// Create from MDItem.
    pub fn from_mditem(item: &MDItem) -> Option<Self> {
        let path = item.path()?.to_string_lossy().to_string();
        let name = item.display_name().unwrap_or_else(|| {
            PathBuf::from(&path)
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default()
        });

        Some(Self {
            path,
            name,
            kind: None,     // Would need attribute lookup
            size: None,     // Would need attribute lookup
            modified: None, // Would need attribute lookup
            created: None,  // Would need attribute lookup
        })
    }
}

/// Parse scope string to MDQueryScope.
fn parse_scope(scope: &str) -> MDQueryScope {
    match scope.to_lowercase().as_str() {
        "home" => MDQueryScope::Home,
        "computer" => MDQueryScope::Computer,
        "network" => MDQueryScope::Network,
        "all" | "indexed" => MDQueryScope::AllIndexed,
        _ => {
            // Treat as custom path
            MDQueryScope::Custom(PathBuf::from(scope))
        }
    }
}

/// Execute a raw Spotlight query string.
pub fn search_raw(query: &str, scope: Option<&str>, limit: u32) -> Result<Vec<SearchResult>> {
    let scopes = scope
        .map(|s| vec![parse_scope(s)])
        .unwrap_or_else(|| vec![MDQueryScope::Home]);

    let md_query = MDQuery::new(query, Some(scopes), Some(limit as usize))
        .map_err(|e| anyhow!("Failed to create query: {}", e))?;

    let items = md_query
        .execute()
        .map_err(|e| anyhow!("Query execution failed: {}", e))?;

    let results: Vec<SearchResult> = items
        .iter()
        .filter_map(|item| SearchResult::from_mditem(item))
        .collect();

    Ok(results)
}

/// Search by file name (substring match).
pub fn search_by_name(name: &str, scope: Option<&str>, limit: u32) -> Result<Vec<SearchResult>> {
    let scopes = scope
        .map(|s| vec![parse_scope(s)])
        .unwrap_or_else(|| vec![MDQueryScope::Home]);

    let md_query = MDQueryBuilder::default()
        .name_like(name)
        .build(scopes, Some(limit as usize))
        .map_err(|e| anyhow!("Failed to build query: {}", e))?;

    let items = md_query
        .execute()
        .map_err(|e| anyhow!("Query execution failed: {}", e))?;

    let results: Vec<SearchResult> = items
        .iter()
        .filter_map(|item| SearchResult::from_mditem(item))
        .collect();

    Ok(results)
}

/// Search by file extension.
pub fn search_by_extension(ext: &str, scope: Option<&str>, limit: u32) -> Result<Vec<SearchResult>> {
    let scopes = scope
        .map(|s| vec![parse_scope(s)])
        .unwrap_or_else(|| vec![MDQueryScope::Home]);

    // Remove leading dot if present
    let ext = ext.trim_start_matches('.');

    let md_query = MDQueryBuilder::default()
        .extension(ext)
        .build(scopes, Some(limit as usize))
        .map_err(|e| anyhow!("Failed to build query: {}", e))?;

    let items = md_query
        .execute()
        .map_err(|e| anyhow!("Query execution failed: {}", e))?;

    let results: Vec<SearchResult> = items
        .iter()
        .filter_map(|item| SearchResult::from_mditem(item))
        .collect();

    Ok(results)
}

/// Search by content type (UTI).
pub fn search_by_content_type(
    content_type: &str,
    scope: Option<&str>,
    limit: u32,
) -> Result<Vec<SearchResult>> {
    let scopes = scope
        .map(|s| vec![parse_scope(s)])
        .unwrap_or_else(|| vec![MDQueryScope::Home]);

    let md_query = MDQueryBuilder::default()
        .content_type(content_type)
        .build(scopes, Some(limit as usize))
        .map_err(|e| anyhow!("Failed to build query: {}", e))?;

    let items = md_query
        .execute()
        .map_err(|e| anyhow!("Query execution failed: {}", e))?;

    let results: Vec<SearchResult> = items
        .iter()
        .filter_map(|item| SearchResult::from_mditem(item))
        .collect();

    Ok(results)
}

/// Find recently modified files.
pub fn search_recent(days: u32, scope: Option<&str>, limit: u32) -> Result<Vec<SearchResult>> {
    let scopes = scope
        .map(|s| vec![parse_scope(s)])
        .unwrap_or_else(|| vec![MDQueryScope::Home]);

    let cutoff = Utc::now() - Duration::days(days as i64);
    let timestamp = cutoff.timestamp();

    let md_query = MDQueryBuilder::default()
        .time(MDItemKey::ModificationDate, MDQueryCompareOp::GreaterThan, timestamp)
        .build(scopes, Some(limit as usize))
        .map_err(|e| anyhow!("Failed to build query: {}", e))?;

    let items = md_query
        .execute()
        .map_err(|e| anyhow!("Query execution failed: {}", e))?;

    let results: Vec<SearchResult> = items
        .iter()
        .filter_map(|item| SearchResult::from_mditem(item))
        .collect();

    Ok(results)
}

/// Find applications.
pub fn search_apps(name: Option<&str>, limit: u32) -> Result<Vec<SearchResult>> {
    let scopes = vec![MDQueryScope::Computer];

    let mut builder = MDQueryBuilder::default().is_app();

    if let Some(n) = name {
        builder = builder.name_like(n);
    }

    let md_query = builder
        .build(scopes, Some(limit as usize))
        .map_err(|e| anyhow!("Failed to build query: {}", e))?;

    let items = md_query
        .execute()
        .map_err(|e| anyhow!("Query execution failed: {}", e))?;

    let results: Vec<SearchResult> = items
        .iter()
        .filter_map(|item| SearchResult::from_mditem(item))
        .collect();

    Ok(results)
}

/// Find directories.
pub fn search_directories(
    name: Option<&str>,
    scope: Option<&str>,
    limit: u32,
) -> Result<Vec<SearchResult>> {
    let scopes = scope
        .map(|s| vec![parse_scope(s)])
        .unwrap_or_else(|| vec![MDQueryScope::Home]);

    let mut builder = MDQueryBuilder::default().is_dir(true);

    if let Some(n) = name {
        builder = builder.name_like(n);
    }

    let md_query = builder
        .build(scopes, Some(limit as usize))
        .map_err(|e| anyhow!("Failed to build query: {}", e))?;

    let items = md_query
        .execute()
        .map_err(|e| anyhow!("Query execution failed: {}", e))?;

    let results: Vec<SearchResult> = items
        .iter()
        .filter_map(|item| SearchResult::from_mditem(item))
        .collect();

    Ok(results)
}

/// Common content type mappings.
pub fn content_type_for_kind(kind: &str) -> Option<&'static str> {
    match kind.to_lowercase().as_str() {
        "pdf" => Some("com.adobe.pdf"),
        "image" | "images" => Some("public.image"),
        "video" | "videos" => Some("public.movie"),
        "audio" | "music" => Some("public.audio"),
        "document" | "documents" => Some("public.content"),
        "text" => Some("public.text"),
        "source" | "code" => Some("public.source-code"),
        "folder" | "directory" => Some("public.folder"),
        "app" | "application" => Some("com.apple.application-bundle"),
        "archive" | "zip" => Some("public.archive"),
        "presentation" => Some("public.presentation"),
        "spreadsheet" => Some("public.spreadsheet"),
        _ => None,
    }
}

/// Search by friendly kind name.
pub fn search_by_kind(
    kind: &str,
    name: Option<&str>,
    scope: Option<&str>,
    limit: u32,
) -> Result<Vec<SearchResult>> {
    let content_type = content_type_for_kind(kind)
        .ok_or_else(|| anyhow!("Unknown kind: {}. Try: pdf, image, video, audio, document, text, source, folder, app, archive", kind))?;

    let scopes = scope
        .map(|s| vec![parse_scope(s)])
        .unwrap_or_else(|| vec![MDQueryScope::Home]);

    let mut builder = MDQueryBuilder::default().content_type(content_type);

    if let Some(n) = name {
        builder = builder.name_like(n);
    }

    let md_query = builder
        .build(scopes, Some(limit as usize))
        .map_err(|e| anyhow!("Failed to build query: {}", e))?;

    let items = md_query
        .execute()
        .map_err(|e| anyhow!("Query execution failed: {}", e))?;

    let results: Vec<SearchResult> = items
        .iter()
        .filter_map(|item| SearchResult::from_mditem(item))
        .collect();

    Ok(results)
}

/// Search by file size range.
pub fn search_by_size(
    min_bytes: Option<u64>,
    max_bytes: Option<u64>,
    scope: Option<&str>,
    limit: u32,
) -> Result<Vec<SearchResult>> {
    // Handle size range queries - mdquery-rs builder doesn't support multiple conditions
    // so we use raw query strings for all size filters
    match (min_bytes, max_bytes) {
        (Some(min), Some(max)) => {
            let query = format!("kMDItemFSSize > {} && kMDItemFSSize < {}", min, max);
            return search_raw(&query, scope, limit);
        }
        (Some(min), None) => {
            let query = format!("kMDItemFSSize > {}", min);
            return search_raw(&query, scope, limit);
        }
        (None, Some(max)) => {
            let query = format!("kMDItemFSSize < {}", max);
            return search_raw(&query, scope, limit);
        }
        (None, None) => {
            // No size filter - return error since this endpoint requires at least one bound
            Err(anyhow!("At least one of min_bytes or max_bytes must be provided"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_type_for_kind() {
        assert_eq!(content_type_for_kind("pdf"), Some("com.adobe.pdf"));
        assert_eq!(content_type_for_kind("image"), Some("public.image"));
        assert_eq!(content_type_for_kind("unknown"), None);
    }

    #[test]
    fn test_search_by_size_requires_bounds() {
        let result = search_by_size(None, None, None, 10);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("At least one of min_bytes"));
    }

    #[test]
    fn test_search_by_size_accepts_empty_scope() {
        let result = search_by_size(Some(1), None, Some(""), 10);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_scope_variants() {
        assert!(matches!(parse_scope("home"), MDQueryScope::Home));
        assert!(matches!(parse_scope("computer"), MDQueryScope::Computer));
        assert!(matches!(parse_scope("network"), MDQueryScope::Network));
        assert!(matches!(parse_scope("all"), MDQueryScope::AllIndexed));
        assert!(matches!(parse_scope("indexed"), MDQueryScope::AllIndexed));

        let custom = parse_scope("/tmp");
        match custom {
            MDQueryScope::Custom(path) => {
                assert!(path.ends_with("tmp"));
            }
            _ => panic!("expected custom scope"),
        }
    }

    #[test]
    fn test_search_by_kind_rejects_unknown() {
        let result = search_by_kind("unknown", None, None, 5);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unknown kind"));
    }
}
