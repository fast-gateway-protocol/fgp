//! FGP service implementation for Discord.
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

use crate::api::DiscordClient;
use crate::models::Embed;

/// FGP service for Discord operations.
pub struct DiscordService {
    client: Arc<DiscordClient>,
    runtime: Runtime,
}

impl DiscordService {
    /// Create a new DiscordService.
    pub fn new() -> Result<Self> {
        let client = DiscordClient::new()?;
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
    // Guild operations
    // ========================================================================

    fn get_current_user(&self, _params: HashMap<String, Value>) -> Result<Value> {
        let client = self.client.clone();

        let result = self.runtime.block_on(async move {
            client.get_current_user().await
        })?;

        Ok(json!(result))
    }

    fn get_guilds(&self, _params: HashMap<String, Value>) -> Result<Value> {
        let client = self.client.clone();

        let result = self.runtime.block_on(async move {
            client.get_guilds().await
        })?;

        Ok(json!({
            "guilds": result,
            "count": result.len()
        }))
    }

    fn get_guild(&self, params: HashMap<String, Value>) -> Result<Value> {
        let guild_id = Self::get_str(&params, "guild_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: guild_id"))?;

        let client = self.client.clone();
        let guild_id = guild_id.to_string();

        let result = self.runtime.block_on(async move {
            client.get_guild(&guild_id).await
        })?;

        Ok(json!(result))
    }

    fn get_channels(&self, params: HashMap<String, Value>) -> Result<Value> {
        let guild_id = Self::get_str(&params, "guild_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: guild_id"))?;

        let client = self.client.clone();
        let guild_id = guild_id.to_string();

        let result = self.runtime.block_on(async move {
            client.get_guild_channels(&guild_id).await
        })?;

        Ok(json!({
            "channels": result,
            "count": result.len()
        }))
    }

    fn get_channel(&self, params: HashMap<String, Value>) -> Result<Value> {
        let channel_id = Self::get_str(&params, "channel_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: channel_id"))?;

        let client = self.client.clone();
        let channel_id = channel_id.to_string();

        let result = self.runtime.block_on(async move {
            client.get_channel(&channel_id).await
        })?;

        Ok(json!(result))
    }

    // ========================================================================
    // Message operations
    // ========================================================================

    fn get_messages(&self, params: HashMap<String, Value>) -> Result<Value> {
        let channel_id = Self::get_str(&params, "channel_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: channel_id"))?;
        let limit = Self::get_i64(&params, "limit");

        let client = self.client.clone();
        let channel_id = channel_id.to_string();

        let result = self.runtime.block_on(async move {
            client.get_messages(&channel_id, limit).await
        })?;

        Ok(json!({
            "messages": result,
            "count": result.len()
        }))
    }

    fn send_message(&self, params: HashMap<String, Value>) -> Result<Value> {
        let channel_id = Self::get_str(&params, "channel_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: channel_id"))?;
        let content = Self::get_str(&params, "content")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: content"))?;

        // Parse embeds if provided
        let embeds: Option<Vec<Embed>> = params.get("embeds")
            .and_then(|v| serde_json::from_value(v.clone()).ok());

        let client = self.client.clone();
        let channel_id = channel_id.to_string();
        let content = content.to_string();

        let result = self.runtime.block_on(async move {
            client.send_message(&channel_id, &content, embeds).await
        })?;

        Ok(json!(result))
    }

    fn edit_message(&self, params: HashMap<String, Value>) -> Result<Value> {
        let channel_id = Self::get_str(&params, "channel_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: channel_id"))?;
        let message_id = Self::get_str(&params, "message_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: message_id"))?;
        let content = Self::get_str(&params, "content")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: content"))?;

        let client = self.client.clone();
        let channel_id = channel_id.to_string();
        let message_id = message_id.to_string();
        let content = content.to_string();

        let result = self.runtime.block_on(async move {
            client.edit_message(&channel_id, &message_id, &content).await
        })?;

        Ok(json!(result))
    }

    fn delete_message(&self, params: HashMap<String, Value>) -> Result<Value> {
        let channel_id = Self::get_str(&params, "channel_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: channel_id"))?;
        let message_id = Self::get_str(&params, "message_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: message_id"))?;

        let client = self.client.clone();
        let channel_id = channel_id.to_string();
        let message_id = message_id.to_string();
        let message_id_clone = message_id.clone();

        self.runtime.block_on(async move {
            client.delete_message(&channel_id, &message_id_clone).await
        })?;

        Ok(json!({"deleted": true, "message_id": message_id}))
    }

    fn add_reaction(&self, params: HashMap<String, Value>) -> Result<Value> {
        let channel_id = Self::get_str(&params, "channel_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: channel_id"))?;
        let message_id = Self::get_str(&params, "message_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: message_id"))?;
        let emoji = Self::get_str(&params, "emoji")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: emoji"))?;

        let client = self.client.clone();
        let channel_id = channel_id.to_string();
        let message_id = message_id.to_string();
        let emoji = emoji.to_string();
        let emoji_clone = emoji.clone();

        self.runtime.block_on(async move {
            client.add_reaction(&channel_id, &message_id, &emoji_clone).await
        })?;

        Ok(json!({"added": true, "emoji": emoji}))
    }

    fn reply(&self, params: HashMap<String, Value>) -> Result<Value> {
        let channel_id = Self::get_str(&params, "channel_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: channel_id"))?;
        let message_id = Self::get_str(&params, "message_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: message_id"))?;
        let content = Self::get_str(&params, "content")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: content"))?;

        let client = self.client.clone();
        let channel_id = channel_id.to_string();
        let message_id = message_id.to_string();
        let content = content.to_string();

        let result = self.runtime.block_on(async move {
            client.reply_to_message(&channel_id, &message_id, &content).await
        })?;

        Ok(json!(result))
    }

    fn get_user(&self, params: HashMap<String, Value>) -> Result<Value> {
        let user_id = Self::get_str(&params, "user_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: user_id"))?;

        let client = self.client.clone();
        let user_id = user_id.to_string();

        let result = self.runtime.block_on(async move {
            client.get_user(&user_id).await
        })?;

        Ok(json!(result))
    }
}

impl FgpService for DiscordService {
    fn name(&self) -> &str {
        "discord"
    }

    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    fn dispatch(&self, method: &str, params: HashMap<String, Value>) -> Result<Value> {
        match method {
            // Bot/User
            "me" | "discord.me" => self.get_current_user(params),
            "user" | "discord.user" => self.get_user(params),

            // Guilds
            "guilds" | "discord.guilds" => self.get_guilds(params),
            "guild" | "discord.guild" => self.get_guild(params),
            "channels" | "discord.channels" => self.get_channels(params),
            "channel" | "discord.channel" => self.get_channel(params),

            // Messages
            "messages" | "discord.messages" => self.get_messages(params),
            "send" | "discord.send" => self.send_message(params),
            "edit" | "discord.edit" => self.edit_message(params),
            "delete" | "discord.delete" => self.delete_message(params),
            "react" | "discord.react" => self.add_reaction(params),
            "reply" | "discord.reply" => self.reply(params),

            _ => anyhow::bail!("Unknown method: {}", method),
        }
    }

    fn method_list(&self) -> Vec<MethodInfo> {
        vec![
            // Bot/User
            MethodInfo::new("discord.me", "Get current bot user"),
            MethodInfo::new("discord.user", "Get user by ID")
                .schema(
                    SchemaBuilder::object()
                        .property("user_id", SchemaBuilder::string().description("User ID"))
                        .required(&["user_id"])
                        .build(),
                ),

            // Guilds
            MethodInfo::new("discord.guilds", "List guilds the bot is in"),
            MethodInfo::new("discord.guild", "Get guild by ID")
                .schema(
                    SchemaBuilder::object()
                        .property("guild_id", SchemaBuilder::string().description("Guild ID"))
                        .required(&["guild_id"])
                        .build(),
                ),
            MethodInfo::new("discord.channels", "List channels in a guild")
                .schema(
                    SchemaBuilder::object()
                        .property("guild_id", SchemaBuilder::string().description("Guild ID"))
                        .required(&["guild_id"])
                        .build(),
                ),
            MethodInfo::new("discord.channel", "Get channel by ID")
                .schema(
                    SchemaBuilder::object()
                        .property("channel_id", SchemaBuilder::string().description("Channel ID"))
                        .required(&["channel_id"])
                        .build(),
                ),

            // Messages
            MethodInfo::new("discord.messages", "Get messages from a channel")
                .schema(
                    SchemaBuilder::object()
                        .property("channel_id", SchemaBuilder::string().description("Channel ID"))
                        .property("limit", SchemaBuilder::integer().description("Number of messages (max 100)"))
                        .required(&["channel_id"])
                        .build(),
                ),
            MethodInfo::new("discord.send", "Send a message to a channel")
                .schema(
                    SchemaBuilder::object()
                        .property("channel_id", SchemaBuilder::string().description("Channel ID"))
                        .property("content", SchemaBuilder::string().description("Message content"))
                        .property("embeds", SchemaBuilder::array().description("Optional embeds"))
                        .required(&["channel_id", "content"])
                        .build(),
                )
                .example("Send message", json!({
                    "channel_id": "123456789",
                    "content": "Hello, Discord!"
                })),
            MethodInfo::new("discord.edit", "Edit a message")
                .schema(
                    SchemaBuilder::object()
                        .property("channel_id", SchemaBuilder::string().description("Channel ID"))
                        .property("message_id", SchemaBuilder::string().description("Message ID"))
                        .property("content", SchemaBuilder::string().description("New content"))
                        .required(&["channel_id", "message_id", "content"])
                        .build(),
                ),
            MethodInfo::new("discord.delete", "Delete a message")
                .schema(
                    SchemaBuilder::object()
                        .property("channel_id", SchemaBuilder::string().description("Channel ID"))
                        .property("message_id", SchemaBuilder::string().description("Message ID"))
                        .required(&["channel_id", "message_id"])
                        .build(),
                ),
            MethodInfo::new("discord.react", "Add a reaction to a message")
                .schema(
                    SchemaBuilder::object()
                        .property("channel_id", SchemaBuilder::string().description("Channel ID"))
                        .property("message_id", SchemaBuilder::string().description("Message ID"))
                        .property("emoji", SchemaBuilder::string().description("Emoji (unicode or custom)"))
                        .required(&["channel_id", "message_id", "emoji"])
                        .build(),
                ),
            MethodInfo::new("discord.reply", "Reply to a message")
                .schema(
                    SchemaBuilder::object()
                        .property("channel_id", SchemaBuilder::string().description("Channel ID"))
                        .property("message_id", SchemaBuilder::string().description("Message ID to reply to"))
                        .property("content", SchemaBuilder::string().description("Reply content"))
                        .required(&["channel_id", "message_id", "content"])
                        .build(),
                ),
        ]
    }

    fn health_check(&self) -> HashMap<String, HealthStatus> {
        let mut checks = HashMap::new();

        checks.insert(
            "bot_token".into(),
            if self.client.has_token() {
                HealthStatus::healthy()
            } else {
                HealthStatus::unhealthy("DISCORD_BOT_TOKEN not set")
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
            std::env::set_var("DISCORD_BOT_TOKEN", "test-token");
        });
    }

    fn test_service() -> DiscordService {
        ensure_env();
        DiscordService::new().expect("service")
    }

    #[test]
    fn test_param_helpers() {
        let mut params = HashMap::new();
        params.insert("guild_id".to_string(), Value::String("guild".to_string()));
        params.insert("limit".to_string(), Value::from(42));

        assert_eq!(DiscordService::get_str(&params, "guild_id"), Some("guild"));
        assert_eq!(DiscordService::get_str(&params, "missing"), None);
        assert_eq!(DiscordService::get_i64(&params, "limit"), Some(42));
        assert_eq!(DiscordService::get_i64(&params, "missing"), None);
    }

    #[test]
    fn test_method_list_required_fields() {
        let service = test_service();
        let methods = service.method_list();

        let send = methods
            .iter()
            .find(|m| m.name == "discord.send")
            .expect("discord.send");
        let send_schema = send.schema.as_ref().expect("schema");
        let required = send_schema
            .get("required")
            .and_then(Value::as_array)
            .expect("required");
        assert!(required.iter().any(|v| v == "channel_id"));
        assert!(required.iter().any(|v| v == "content"));

        let channel = methods
            .iter()
            .find(|m| m.name == "discord.channel")
            .expect("discord.channel");
        let channel_schema = channel.schema.as_ref().expect("schema");
        let required = channel_schema
            .get("required")
            .and_then(Value::as_array)
            .expect("required");
        assert!(required.iter().any(|v| v == "channel_id"));
    }

    #[test]
    fn test_dispatch_unknown_method() {
        let service = test_service();
        let result = service.dispatch("discord.unknown", HashMap::new());
        assert!(result.is_err());
    }
}
