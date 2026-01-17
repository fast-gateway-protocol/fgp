//! FGP service implementation for YouTube.
//!
//! # CHANGELOG (recent first, max 5 entries)
//! 01/15/2026 - Initial implementation (Claude)

use anyhow::Result;
use fgp_daemon::schema::SchemaBuilder;
use fgp_daemon::service::{HealthStatus, MethodInfo};
use fgp_daemon::FgpService;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Runtime;

use crate::api::YouTubeClient;

/// FGP service for YouTube operations.
pub struct YouTubeService {
    client: Arc<YouTubeClient>,
    runtime: Runtime,
}

impl YouTubeService {
    /// Create a new YouTubeService.
    pub fn new() -> Result<Self> {
        let client = YouTubeClient::new()?;
        let runtime = Runtime::new()?;

        Ok(Self {
            client: Arc::new(client),
            runtime,
        })
    }

    /// Helper to get a string parameter.
    fn get_str<'a>(params: &'a HashMap<String, Value>, key: &str) -> Option<&'a str> {
        params.get(key).and_then(|v| v.as_str())
    }

    /// Helper to get an i64 parameter.
    fn get_i64(params: &HashMap<String, Value>, key: &str) -> Option<i64> {
        params.get(key).and_then(|v| v.as_i64())
    }

    // ========================================================================
    // Search
    // ========================================================================

    fn search(&self, params: HashMap<String, Value>) -> Result<Value> {
        let query = Self::get_str(&params, "query")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: query"))?;
        let search_type = Self::get_str(&params, "type");
        let max_results = Self::get_i64(&params, "max_results");
        let page_token = Self::get_str(&params, "page_token");

        let client = self.client.clone();
        let query = query.to_string();
        let search_type = search_type.map(|s| s.to_string());
        let page_token = page_token.map(|s| s.to_string());

        let result = self.runtime.block_on(async move {
            client.search(&query, search_type.as_deref(), max_results, page_token.as_deref()).await
        })?;

        Ok(json!(result))
    }

    // ========================================================================
    // Videos
    // ========================================================================

    fn get_video(&self, params: HashMap<String, Value>) -> Result<Value> {
        let video_id = Self::get_str(&params, "video_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: video_id"))?;

        let client = self.client.clone();
        let video_id = video_id.to_string();

        let result = self.runtime.block_on(async move {
            client.get_video(&video_id).await
        })?;

        Ok(json!(result))
    }

    fn get_videos(&self, params: HashMap<String, Value>) -> Result<Value> {
        let video_ids = params.get("video_ids")
            .and_then(|v| v.as_array())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: video_ids"))?;

        let video_ids: Vec<String> = video_ids.iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();

        let client = self.client.clone();

        let result = self.runtime.block_on(async move {
            let refs: Vec<&str> = video_ids.iter().map(|s| s.as_str()).collect();
            client.get_videos(&refs).await
        })?;

        Ok(json!(result))
    }

    // ========================================================================
    // Channels
    // ========================================================================

    fn get_channel(&self, params: HashMap<String, Value>) -> Result<Value> {
        let channel_id = Self::get_str(&params, "channel_id");
        let username = Self::get_str(&params, "username");

        let client = self.client.clone();

        let result = self.runtime.block_on(async move {
            if let Some(id) = channel_id {
                client.get_channel(id).await
            } else if let Some(name) = username {
                client.get_channel_by_username(name).await
            } else {
                anyhow::bail!("Missing channel_id or username parameter")
            }
        })?;

        Ok(json!(result))
    }

    // ========================================================================
    // Playlists
    // ========================================================================

    fn get_playlists(&self, params: HashMap<String, Value>) -> Result<Value> {
        let channel_id = Self::get_str(&params, "channel_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: channel_id"))?;
        let max_results = Self::get_i64(&params, "max_results");

        let client = self.client.clone();
        let channel_id = channel_id.to_string();

        let result = self.runtime.block_on(async move {
            client.get_playlists(&channel_id, max_results).await
        })?;

        Ok(json!(result))
    }

    fn get_playlist_items(&self, params: HashMap<String, Value>) -> Result<Value> {
        let playlist_id = Self::get_str(&params, "playlist_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: playlist_id"))?;
        let max_results = Self::get_i64(&params, "max_results");
        let page_token = Self::get_str(&params, "page_token");

        let client = self.client.clone();
        let playlist_id = playlist_id.to_string();
        let page_token = page_token.map(|s| s.to_string());

        let result = self.runtime.block_on(async move {
            client.get_playlist_items(&playlist_id, max_results, page_token.as_deref()).await
        })?;

        Ok(json!(result))
    }

    // ========================================================================
    // Comments
    // ========================================================================

    fn get_comments(&self, params: HashMap<String, Value>) -> Result<Value> {
        let video_id = Self::get_str(&params, "video_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: video_id"))?;
        let max_results = Self::get_i64(&params, "max_results");
        let page_token = Self::get_str(&params, "page_token");

        let client = self.client.clone();
        let video_id = video_id.to_string();
        let page_token = page_token.map(|s| s.to_string());

        let result = self.runtime.block_on(async move {
            client.get_comments(&video_id, max_results, page_token.as_deref()).await
        })?;

        Ok(json!(result))
    }
}

impl FgpService for YouTubeService {
    fn name(&self) -> &str {
        "youtube"
    }

    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    fn dispatch(&self, method: &str, params: HashMap<String, Value>) -> Result<Value> {
        match method {
            // Search
            "search" | "youtube.search" => self.search(params),

            // Videos
            "video" | "youtube.video" => self.get_video(params),
            "videos" | "youtube.videos" => self.get_videos(params),

            // Channels
            "channel" | "youtube.channel" => self.get_channel(params),

            // Playlists
            "playlists" | "youtube.playlists" => self.get_playlists(params),
            "playlist_items" | "youtube.playlist_items" => self.get_playlist_items(params),

            // Comments
            "comments" | "youtube.comments" => self.get_comments(params),

            _ => anyhow::bail!("Unknown method: {}", method),
        }
    }

    fn method_list(&self) -> Vec<MethodInfo> {
        vec![
            // Search
            MethodInfo::new("youtube.search", "Search for videos, channels, or playlists")
                .schema(
                    SchemaBuilder::object()
                        .property("query", SchemaBuilder::string().description("Search query"))
                        .property("type", SchemaBuilder::string().description("video, channel, or playlist"))
                        .property("max_results", SchemaBuilder::integer().description("Max results (1-50)"))
                        .property("page_token", SchemaBuilder::string().description("Pagination token"))
                        .required(&["query"])
                        .build(),
                )
                .example("Search videos", json!({"query": "rust programming", "type": "video", "max_results": 10})),

            // Videos
            MethodInfo::new("youtube.video", "Get video details")
                .schema(
                    SchemaBuilder::object()
                        .property("video_id", SchemaBuilder::string().description("Video ID"))
                        .required(&["video_id"])
                        .build(),
                ),
            MethodInfo::new("youtube.videos", "Get multiple videos")
                .schema(
                    SchemaBuilder::object()
                        .property("video_ids", SchemaBuilder::array().description("List of video IDs"))
                        .required(&["video_ids"])
                        .build(),
                ),

            // Channels
            MethodInfo::new("youtube.channel", "Get channel details")
                .schema(
                    SchemaBuilder::object()
                        .property("channel_id", SchemaBuilder::string().description("Channel ID"))
                        .property("username", SchemaBuilder::string().description("Channel username"))
                        .build(),
                ),

            // Playlists
            MethodInfo::new("youtube.playlists", "Get playlists for a channel")
                .schema(
                    SchemaBuilder::object()
                        .property("channel_id", SchemaBuilder::string().description("Channel ID"))
                        .property("max_results", SchemaBuilder::integer().description("Max results (1-50)"))
                        .required(&["channel_id"])
                        .build(),
                ),
            MethodInfo::new("youtube.playlist_items", "Get items in a playlist")
                .schema(
                    SchemaBuilder::object()
                        .property("playlist_id", SchemaBuilder::string().description("Playlist ID"))
                        .property("max_results", SchemaBuilder::integer().description("Max results (1-50)"))
                        .property("page_token", SchemaBuilder::string().description("Pagination token"))
                        .required(&["playlist_id"])
                        .build(),
                ),

            // Comments
            MethodInfo::new("youtube.comments", "Get comments on a video")
                .schema(
                    SchemaBuilder::object()
                        .property("video_id", SchemaBuilder::string().description("Video ID"))
                        .property("max_results", SchemaBuilder::integer().description("Max results (1-100)"))
                        .property("page_token", SchemaBuilder::string().description("Pagination token"))
                        .required(&["video_id"])
                        .build(),
                ),
        ]
    }

    fn health_check(&self) -> HashMap<String, HealthStatus> {
        let mut checks = HashMap::new();

        checks.insert(
            "api_key".into(),
            if self.client.has_key() {
                HealthStatus::healthy()
            } else {
                HealthStatus::unhealthy("YOUTUBE_API_KEY not set")
            },
        );

        checks
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn ensure_env() {
        INIT.call_once(|| {
            std::env::set_var("YOUTUBE_API_KEY", "test-key");
        });
    }

    fn test_service() -> YouTubeService {
        ensure_env();
        YouTubeService::new().expect("service")
    }

    #[test]
    fn test_param_helpers() {
        let mut params = HashMap::new();
        params.insert("query".to_string(), Value::String("rust".to_string()));
        params.insert("max_results".to_string(), Value::from(25));

        assert_eq!(YouTubeService::get_str(&params, "query"), Some("rust"));
        assert_eq!(YouTubeService::get_str(&params, "missing"), None);
        assert_eq!(YouTubeService::get_i64(&params, "max_results"), Some(25));
        assert_eq!(YouTubeService::get_i64(&params, "missing"), None);
    }

    #[test]
    fn test_method_list_required_fields() {
        let service = test_service();
        let methods = service.method_list();

        let search = methods
            .iter()
            .find(|m| m.name == "youtube.search")
            .expect("youtube.search");
        let search_schema = search.schema.as_ref().expect("schema");
        let required = search_schema
            .get("required")
            .and_then(Value::as_array)
            .expect("required");
        assert!(required.iter().any(|v| v == "query"));

        let video = methods
            .iter()
            .find(|m| m.name == "youtube.video")
            .expect("youtube.video");
        let video_schema = video.schema.as_ref().expect("schema");
        let required = video_schema
            .get("required")
            .and_then(Value::as_array)
            .expect("required");
        assert!(required.iter().any(|v| v == "video_id"));
    }

    #[test]
    fn test_dispatch_unknown_method() {
        let service = test_service();
        let result = service.dispatch("youtube.unknown", HashMap::new());
        assert!(result.is_err());
    }
}
