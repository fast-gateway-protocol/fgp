//! Data models for YouTube Data API v3.
//!
//! # CHANGELOG (recent first, max 5 entries)
//! 01/15/2026 - Added Default derives for serde deserialization (Claude)
//! 01/15/2026 - Initial implementation (Claude)

use serde::{Deserialize, Serialize};

/// YouTube API list response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResponse<T> {
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub etag: Option<String>,
    #[serde(default, rename = "nextPageToken")]
    pub next_page_token: Option<String>,
    #[serde(default, rename = "prevPageToken")]
    pub prev_page_token: Option<String>,
    #[serde(default, rename = "pageInfo")]
    pub page_info: Option<PageInfo>,
    #[serde(default)]
    pub items: Vec<T>,
}

/// Pagination info.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PageInfo {
    #[serde(default, rename = "totalResults")]
    pub total_results: Option<i64>,
    #[serde(default, rename = "resultsPerPage")]
    pub results_per_page: Option<i64>,
}

/// A search result.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchResult {
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub etag: Option<String>,
    #[serde(default)]
    pub id: SearchResultId,
    #[serde(default)]
    pub snippet: Option<VideoSnippet>,
}

/// Search result ID (can be video, channel, or playlist).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchResultId {
    #[serde(default)]
    pub kind: String,
    #[serde(default, rename = "videoId")]
    pub video_id: Option<String>,
    #[serde(default, rename = "channelId")]
    pub channel_id: Option<String>,
    #[serde(default, rename = "playlistId")]
    pub playlist_id: Option<String>,
}

/// A video resource.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Video {
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub etag: Option<String>,
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub snippet: Option<VideoSnippet>,
    #[serde(default)]
    pub statistics: Option<VideoStatistics>,
    #[serde(default, rename = "contentDetails")]
    pub content_details: Option<VideoContentDetails>,
}

/// Video snippet (metadata).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VideoSnippet {
    #[serde(default, rename = "publishedAt")]
    pub published_at: Option<String>,
    #[serde(default, rename = "channelId")]
    pub channel_id: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub thumbnails: Option<Thumbnails>,
    #[serde(default, rename = "channelTitle")]
    pub channel_title: Option<String>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    #[serde(default, rename = "categoryId")]
    pub category_id: Option<String>,
}

/// Video thumbnails.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Thumbnails {
    #[serde(default)]
    pub default: Option<Thumbnail>,
    #[serde(default)]
    pub medium: Option<Thumbnail>,
    #[serde(default)]
    pub high: Option<Thumbnail>,
    #[serde(default)]
    pub standard: Option<Thumbnail>,
    #[serde(default)]
    pub maxres: Option<Thumbnail>,
}

/// A single thumbnail.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Thumbnail {
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub width: Option<i64>,
    #[serde(default)]
    pub height: Option<i64>,
}

/// Video statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VideoStatistics {
    #[serde(default, rename = "viewCount")]
    pub view_count: Option<String>,
    #[serde(default, rename = "likeCount")]
    pub like_count: Option<String>,
    #[serde(default, rename = "commentCount")]
    pub comment_count: Option<String>,
}

/// Video content details.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VideoContentDetails {
    #[serde(default)]
    pub duration: Option<String>,
    #[serde(default)]
    pub dimension: Option<String>,
    #[serde(default)]
    pub definition: Option<String>,
    #[serde(default)]
    pub caption: Option<String>,
}

/// A channel resource.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Channel {
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub etag: Option<String>,
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub snippet: Option<ChannelSnippet>,
    #[serde(default)]
    pub statistics: Option<ChannelStatistics>,
}

/// Channel snippet.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChannelSnippet {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default, rename = "customUrl")]
    pub custom_url: Option<String>,
    #[serde(default, rename = "publishedAt")]
    pub published_at: Option<String>,
    #[serde(default)]
    pub thumbnails: Option<Thumbnails>,
    #[serde(default)]
    pub country: Option<String>,
}

/// Channel statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChannelStatistics {
    #[serde(default, rename = "viewCount")]
    pub view_count: Option<String>,
    #[serde(default, rename = "subscriberCount")]
    pub subscriber_count: Option<String>,
    #[serde(default, rename = "videoCount")]
    pub video_count: Option<String>,
}

/// A playlist resource.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Playlist {
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub etag: Option<String>,
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub snippet: Option<PlaylistSnippet>,
    #[serde(default, rename = "contentDetails")]
    pub content_details: Option<PlaylistContentDetails>,
}

/// Playlist snippet.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlaylistSnippet {
    #[serde(default, rename = "publishedAt")]
    pub published_at: Option<String>,
    #[serde(default, rename = "channelId")]
    pub channel_id: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub thumbnails: Option<Thumbnails>,
    #[serde(default, rename = "channelTitle")]
    pub channel_title: Option<String>,
}

/// Playlist content details.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlaylistContentDetails {
    #[serde(default, rename = "itemCount")]
    pub item_count: Option<i64>,
}

/// A playlist item.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlaylistItem {
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub etag: Option<String>,
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub snippet: Option<PlaylistItemSnippet>,
}

/// Playlist item snippet.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlaylistItemSnippet {
    #[serde(default, rename = "publishedAt")]
    pub published_at: Option<String>,
    #[serde(default, rename = "channelId")]
    pub channel_id: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub thumbnails: Option<Thumbnails>,
    #[serde(default, rename = "channelTitle")]
    pub channel_title: Option<String>,
    #[serde(default, rename = "playlistId")]
    pub playlist_id: Option<String>,
    #[serde(default)]
    pub position: Option<i64>,
    #[serde(default, rename = "resourceId")]
    pub resource_id: Option<ResourceId>,
}

/// Resource ID for playlist items.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceId {
    #[serde(default)]
    pub kind: String,
    #[serde(default, rename = "videoId")]
    pub video_id: Option<String>,
}

/// A comment thread.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommentThread {
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub etag: Option<String>,
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub snippet: Option<CommentThreadSnippet>,
}

/// Comment thread snippet.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommentThreadSnippet {
    #[serde(default, rename = "videoId")]
    pub video_id: Option<String>,
    #[serde(default, rename = "topLevelComment")]
    pub top_level_comment: Option<Comment>,
    #[serde(default, rename = "totalReplyCount")]
    pub total_reply_count: Option<i64>,
}

/// A comment.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Comment {
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub etag: Option<String>,
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub snippet: Option<CommentSnippet>,
}

/// Comment snippet.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommentSnippet {
    #[serde(default, rename = "authorDisplayName")]
    pub author_display_name: Option<String>,
    #[serde(default, rename = "authorChannelUrl")]
    pub author_channel_url: Option<String>,
    #[serde(default, rename = "textDisplay")]
    pub text_display: Option<String>,
    #[serde(default, rename = "textOriginal")]
    pub text_original: Option<String>,
    #[serde(default, rename = "likeCount")]
    pub like_count: Option<i64>,
    #[serde(default, rename = "publishedAt")]
    pub published_at: Option<String>,
    #[serde(default, rename = "updatedAt")]
    pub updated_at: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn list_response_defaults_items() {
        let value = json!({
            "kind": "youtube#searchListResponse"
        });
        let response: ListResponse<SearchResult> =
            serde_json::from_value(value).expect("deserialize list response");

        assert_eq!(response.kind, "youtube#searchListResponse");
        assert!(response.items.is_empty());
        assert!(response.page_info.is_none());
    }

    #[test]
    fn search_result_id_maps_video_id() {
        let value = json!({
            "id": {
                "videoId": "vid_1",
                "kind": "youtube#video"
            }
        });
        let result: SearchResult = serde_json::from_value(value).expect("deserialize result");

        assert_eq!(result.id.video_id.as_deref(), Some("vid_1"));
        assert_eq!(result.id.kind, "youtube#video");
    }

    #[test]
    fn video_snippet_handles_thumbnails() {
        let value = json!({
            "title": "Demo",
            "thumbnails": {
                "high": {
                    "url": "https://example.com/high.jpg",
                    "width": 1280,
                    "height": 720
                }
            }
        });
        let snippet: VideoSnippet = serde_json::from_value(value).expect("deserialize snippet");

        let high = snippet.thumbnails.expect("thumbnails").high.expect("high");
        assert_eq!(high.url, "https://example.com/high.jpg");
        assert_eq!(high.width, Some(1280));
        assert_eq!(high.height, Some(720));
    }

    #[test]
    fn comment_thread_maps_snippet() {
        let value = json!({
            "id": "thread_1",
            "snippet": {
                "videoId": "vid_2",
                "totalReplyCount": 3,
                "topLevelComment": {
                    "id": "comment_1",
                    "snippet": {
                        "authorDisplayName": "Jane",
                        "textOriginal": "Great video!"
                    }
                }
            }
        });
        let thread: CommentThread = serde_json::from_value(value).expect("deserialize thread");

        let snippet = thread.snippet.expect("snippet");
        assert_eq!(snippet.video_id.as_deref(), Some("vid_2"));
        assert_eq!(snippet.total_reply_count, Some(3));
        let comment = snippet.top_level_comment.expect("comment");
        let comment_snippet = comment.snippet.expect("comment snippet");
        assert_eq!(comment_snippet.author_display_name.as_deref(), Some("Jane"));
        assert_eq!(comment_snippet.text_original.as_deref(), Some("Great video!"));
    }
}
