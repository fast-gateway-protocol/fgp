//! Data models for Discord API.
//!
//! # CHANGELOG (recent first, max 5 entries)
//! 01/15/2026 - Initial implementation (Claude)

use serde::{Deserialize, Serialize};

/// A Discord guild (server).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Guild {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub owner_id: Option<String>,
    #[serde(default)]
    pub member_count: Option<i64>,
}

/// A Discord channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub id: String,
    #[serde(rename = "type")]
    pub channel_type: i64,
    #[serde(default)]
    pub guild_id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub topic: Option<String>,
    #[serde(default)]
    pub position: Option<i64>,
    #[serde(default)]
    pub parent_id: Option<String>,
}

/// A Discord user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    #[serde(default)]
    pub discriminator: Option<String>,
    #[serde(default)]
    pub global_name: Option<String>,
    #[serde(default)]
    pub avatar: Option<String>,
    #[serde(default)]
    pub bot: Option<bool>,
}

/// A Discord message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub channel_id: String,
    #[serde(default)]
    pub author: Option<User>,
    pub content: String,
    pub timestamp: String,
    #[serde(default)]
    pub edited_timestamp: Option<String>,
    #[serde(default)]
    pub tts: Option<bool>,
    #[serde(default)]
    pub mention_everyone: Option<bool>,
    #[serde(default)]
    pub mentions: Option<Vec<User>>,
    #[serde(default)]
    pub attachments: Option<Vec<Attachment>>,
    #[serde(default)]
    pub embeds: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub reactions: Option<Vec<Reaction>>,
}

/// A message attachment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub id: String,
    pub filename: String,
    #[serde(default)]
    pub size: Option<i64>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub proxy_url: Option<String>,
    #[serde(default)]
    pub content_type: Option<String>,
}

/// A message reaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reaction {
    pub count: i64,
    pub me: bool,
    pub emoji: Emoji,
}

/// An emoji.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Emoji {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
}

/// An embed for rich messages.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Embed {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub footer: Option<EmbedFooter>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<EmbedField>>,
}

/// Embed footer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedFooter {
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
}

/// Embed field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedField {
    pub name: String,
    pub value: String,
    #[serde(default)]
    pub inline: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn guild_defaults_optional_fields() {
        let value = json!({
            "id": "g1",
            "name": "Server"
        });
        let guild: Guild = serde_json::from_value(value).expect("deserialize guild");

        assert_eq!(guild.id, "g1");
        assert!(guild.icon.is_none());
        assert!(guild.owner_id.is_none());
        assert!(guild.member_count.is_none());
    }

    #[test]
    fn channel_reads_type_field() {
        let value = json!({
            "id": "c1",
            "type": 0,
            "name": "general"
        });
        let channel: Channel = serde_json::from_value(value).expect("deserialize channel");

        assert_eq!(channel.channel_type, 0);
        assert_eq!(channel.name.as_deref(), Some("general"));
    }

    #[test]
    fn message_handles_optional_fields() {
        let value = json!({
            "id": "m1",
            "channel_id": "c1",
            "content": "hello",
            "timestamp": "2026-01-01T00:00:00Z"
        });
        let message: Message = serde_json::from_value(value).expect("deserialize message");

        assert_eq!(message.content, "hello");
        assert!(message.author.is_none());
        assert!(message.mentions.is_none());
        assert!(message.attachments.is_none());
        assert!(message.reactions.is_none());
    }

    #[test]
    fn embed_serializes_minimally() {
        let embed = Embed {
            title: Some("Title".to_string()),
            ..Embed::default()
        };
        let value = serde_json::to_value(embed).expect("serialize embed");
        let object = value.as_object().expect("embed should be object");

        assert_eq!(object.get("title").and_then(|v| v.as_str()), Some("Title"));
        assert!(!object.contains_key("description"));
        assert!(!object.contains_key("fields"));
    }
}
