//! YouTube Data API v3 client with connection pooling.
//!
//! Uses YOUTUBE_API_KEY env var for authentication.
//!
//! # CHANGELOG (recent first, max 5 entries)
//! 01/15/2026 - Initial implementation (Claude)

use anyhow::{bail, Context, Result};
use reqwest::Client;

use crate::models::{
    Channel, CommentThread, ListResponse, Playlist, PlaylistItem, SearchResult, Video,
};

const API_BASE: &str = "https://www.googleapis.com/youtube/v3";

/// YouTube Data API client with persistent connection pooling.
pub struct YouTubeClient {
    client: Client,
    api_key: Option<String>,
}

impl YouTubeClient {
    /// Create a new YouTube client.
    pub fn new() -> Result<Self> {
        let api_key = std::env::var("YOUTUBE_API_KEY").ok();

        let client = Client::builder()
            .pool_max_idle_per_host(5)
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("fgp-youtube/1.0.0")
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self { client, api_key })
    }

    /// Check if API key is available.
    pub fn has_key(&self) -> bool {
        self.api_key.is_some()
    }

    /// Get API key.
    fn get_key(&self) -> Result<&str> {
        self.api_key
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("YOUTUBE_API_KEY not set"))
    }

    // ========================================================================
    // Search
    // ========================================================================

    /// Search for videos, channels, or playlists.
    pub async fn search(
        &self,
        query: &str,
        search_type: Option<&str>, // "video", "channel", "playlist"
        max_results: Option<i64>,
        page_token: Option<&str>,
    ) -> Result<ListResponse<SearchResult>> {
        let api_key = self.get_key()?;

        let mut url = format!(
            "{}/search?part=snippet&q={}&key={}",
            API_BASE,
            urlencoding::encode(query),
            api_key
        );

        if let Some(t) = search_type {
            url.push_str(&format!("&type={}", t));
        }
        if let Some(max) = max_results {
            url.push_str(&format!("&maxResults={}", max.min(50)));
        }
        if let Some(token) = page_token {
            url.push_str(&format!("&pageToken={}", token));
        }

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to search")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("YouTube API error: {} - {}", status, text);
        }

        response.json().await.context("Failed to parse search results")
    }

    // ========================================================================
    // Videos
    // ========================================================================

    /// Get video details.
    pub async fn get_videos(&self, video_ids: &[&str]) -> Result<ListResponse<Video>> {
        let api_key = self.get_key()?;

        let ids = video_ids.join(",");
        let url = format!(
            "{}/videos?part=snippet,statistics,contentDetails&id={}&key={}",
            API_BASE, ids, api_key
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to get videos")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("YouTube API error: {} - {}", status, text);
        }

        response.json().await.context("Failed to parse videos")
    }

    /// Get a single video.
    pub async fn get_video(&self, video_id: &str) -> Result<Video> {
        let result = self.get_videos(&[video_id]).await?;
        result
            .items
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("Video not found"))
    }

    // ========================================================================
    // Channels
    // ========================================================================

    /// Get channel details.
    pub async fn get_channels(&self, channel_ids: &[&str]) -> Result<ListResponse<Channel>> {
        let api_key = self.get_key()?;

        let ids = channel_ids.join(",");
        let url = format!(
            "{}/channels?part=snippet,statistics&id={}&key={}",
            API_BASE, ids, api_key
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to get channels")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("YouTube API error: {} - {}", status, text);
        }

        response.json().await.context("Failed to parse channels")
    }

    /// Get a single channel.
    pub async fn get_channel(&self, channel_id: &str) -> Result<Channel> {
        let result = self.get_channels(&[channel_id]).await?;
        result
            .items
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("Channel not found"))
    }

    /// Get channel by username.
    pub async fn get_channel_by_username(&self, username: &str) -> Result<Channel> {
        let api_key = self.get_key()?;

        let url = format!(
            "{}/channels?part=snippet,statistics&forUsername={}&key={}",
            API_BASE, username, api_key
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to get channel")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("YouTube API error: {} - {}", status, text);
        }

        let result: ListResponse<Channel> = response.json().await.context("Failed to parse channel")?;
        result
            .items
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("Channel not found"))
    }

    // ========================================================================
    // Playlists
    // ========================================================================

    /// Get playlists for a channel.
    pub async fn get_playlists(
        &self,
        channel_id: &str,
        max_results: Option<i64>,
    ) -> Result<ListResponse<Playlist>> {
        let api_key = self.get_key()?;

        let mut url = format!(
            "{}/playlists?part=snippet,contentDetails&channelId={}&key={}",
            API_BASE, channel_id, api_key
        );

        if let Some(max) = max_results {
            url.push_str(&format!("&maxResults={}", max.min(50)));
        }

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to get playlists")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("YouTube API error: {} - {}", status, text);
        }

        response.json().await.context("Failed to parse playlists")
    }

    /// Get playlist items.
    pub async fn get_playlist_items(
        &self,
        playlist_id: &str,
        max_results: Option<i64>,
        page_token: Option<&str>,
    ) -> Result<ListResponse<PlaylistItem>> {
        let api_key = self.get_key()?;

        let mut url = format!(
            "{}/playlistItems?part=snippet&playlistId={}&key={}",
            API_BASE, playlist_id, api_key
        );

        if let Some(max) = max_results {
            url.push_str(&format!("&maxResults={}", max.min(50)));
        }
        if let Some(token) = page_token {
            url.push_str(&format!("&pageToken={}", token));
        }

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to get playlist items")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("YouTube API error: {} - {}", status, text);
        }

        response.json().await.context("Failed to parse playlist items")
    }

    // ========================================================================
    // Comments
    // ========================================================================

    /// Get comment threads for a video.
    pub async fn get_comments(
        &self,
        video_id: &str,
        max_results: Option<i64>,
        page_token: Option<&str>,
    ) -> Result<ListResponse<CommentThread>> {
        let api_key = self.get_key()?;

        let mut url = format!(
            "{}/commentThreads?part=snippet&videoId={}&key={}",
            API_BASE, video_id, api_key
        );

        if let Some(max) = max_results {
            url.push_str(&format!("&maxResults={}", max.min(100)));
        }
        if let Some(token) = page_token {
            url.push_str(&format!("&pageToken={}", token));
        }

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to get comments")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("YouTube API error: {} - {}", status, text);
        }

        response.json().await.context("Failed to parse comments")
    }
}

// URL encoding helper
mod urlencoding {
    pub fn encode(s: &str) -> String {
        let mut result = String::new();
        for c in s.chars() {
            match c {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => result.push(c),
                ' ' => result.push_str("%20"),
                _ => {
                    for byte in c.to_string().as_bytes() {
                        result.push_str(&format!("%{:02X}", byte));
                    }
                }
            }
        }
        result
    }
}
