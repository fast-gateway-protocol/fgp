//! Discord HTTP API client with connection pooling.
//!
//! Uses DISCORD_BOT_TOKEN env var for authentication.
//!
//! # CHANGELOG (recent first, max 5 entries)
//! 01/15/2026 - Initial implementation (Claude)

use anyhow::{bail, Context, Result};
use reqwest::Client;
use serde_json::Value;

use crate::models::{Channel, Embed, Guild, Message, User};

const API_BASE: &str = "https://discord.com/api/v10";

/// Discord API client with persistent connection pooling.
pub struct DiscordClient {
    client: Client,
    bot_token: Option<String>,
}

impl DiscordClient {
    /// Create a new Discord client.
    pub fn new() -> Result<Self> {
        let bot_token = std::env::var("DISCORD_BOT_TOKEN").ok();

        let client = Client::builder()
            .pool_max_idle_per_host(5)
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("DiscordBot (fgp-discord, 1.0.0)")
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self { client, bot_token })
    }

    /// Check if bot token is available.
    pub fn has_token(&self) -> bool {
        self.bot_token.is_some()
    }

    /// Get bot token.
    fn get_token(&self) -> Result<&str> {
        self.bot_token
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("DISCORD_BOT_TOKEN not set"))
    }

    // ========================================================================
    // Guild operations
    // ========================================================================

    /// Get current bot user.
    pub async fn get_current_user(&self) -> Result<User> {
        let token = self.get_token()?;

        let response = self
            .client
            .get(format!("{}/users/@me", API_BASE))
            .header("Authorization", format!("Bot {}", token))
            .send()
            .await
            .context("Failed to get current user")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Discord API error: {} - {}", status, text);
        }

        response.json().await.context("Failed to parse user")
    }

    /// Get guilds the bot is in.
    pub async fn get_guilds(&self) -> Result<Vec<Guild>> {
        let token = self.get_token()?;

        let response = self
            .client
            .get(format!("{}/users/@me/guilds", API_BASE))
            .header("Authorization", format!("Bot {}", token))
            .send()
            .await
            .context("Failed to get guilds")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Discord API error: {} - {}", status, text);
        }

        response.json().await.context("Failed to parse guilds")
    }

    /// Get a guild by ID.
    pub async fn get_guild(&self, guild_id: &str) -> Result<Guild> {
        let token = self.get_token()?;

        let response = self
            .client
            .get(format!("{}/guilds/{}", API_BASE, guild_id))
            .header("Authorization", format!("Bot {}", token))
            .send()
            .await
            .context("Failed to get guild")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Discord API error: {} - {}", status, text);
        }

        response.json().await.context("Failed to parse guild")
    }

    /// Get channels in a guild.
    pub async fn get_guild_channels(&self, guild_id: &str) -> Result<Vec<Channel>> {
        let token = self.get_token()?;

        let response = self
            .client
            .get(format!("{}/guilds/{}/channels", API_BASE, guild_id))
            .header("Authorization", format!("Bot {}", token))
            .send()
            .await
            .context("Failed to get channels")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Discord API error: {} - {}", status, text);
        }

        response.json().await.context("Failed to parse channels")
    }

    // ========================================================================
    // Channel operations
    // ========================================================================

    /// Get a channel by ID.
    pub async fn get_channel(&self, channel_id: &str) -> Result<Channel> {
        let token = self.get_token()?;

        let response = self
            .client
            .get(format!("{}/channels/{}", API_BASE, channel_id))
            .header("Authorization", format!("Bot {}", token))
            .send()
            .await
            .context("Failed to get channel")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Discord API error: {} - {}", status, text);
        }

        response.json().await.context("Failed to parse channel")
    }

    // ========================================================================
    // Message operations
    // ========================================================================

    /// Get messages from a channel.
    pub async fn get_messages(&self, channel_id: &str, limit: Option<i64>) -> Result<Vec<Message>> {
        let token = self.get_token()?;

        let mut url = format!("{}/channels/{}/messages", API_BASE, channel_id);
        if let Some(l) = limit {
            url.push_str(&format!("?limit={}", l.min(100)));
        }

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bot {}", token))
            .send()
            .await
            .context("Failed to get messages")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Discord API error: {} - {}", status, text);
        }

        response.json().await.context("Failed to parse messages")
    }

    /// Send a message to a channel.
    pub async fn send_message(
        &self,
        channel_id: &str,
        content: &str,
        embeds: Option<Vec<Embed>>,
    ) -> Result<Message> {
        let token = self.get_token()?;

        let mut body = serde_json::json!({
            "content": content
        });

        if let Some(e) = embeds {
            body["embeds"] = serde_json::json!(e);
        }

        let response = self
            .client
            .post(format!("{}/channels/{}/messages", API_BASE, channel_id))
            .header("Authorization", format!("Bot {}", token))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Failed to send message")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Discord API error: {} - {}", status, text);
        }

        response.json().await.context("Failed to parse message")
    }

    /// Edit a message.
    pub async fn edit_message(
        &self,
        channel_id: &str,
        message_id: &str,
        content: &str,
    ) -> Result<Message> {
        let token = self.get_token()?;

        let body = serde_json::json!({
            "content": content
        });

        let response = self
            .client
            .patch(format!(
                "{}/channels/{}/messages/{}",
                API_BASE, channel_id, message_id
            ))
            .header("Authorization", format!("Bot {}", token))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Failed to edit message")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Discord API error: {} - {}", status, text);
        }

        response.json().await.context("Failed to parse message")
    }

    /// Delete a message.
    pub async fn delete_message(&self, channel_id: &str, message_id: &str) -> Result<()> {
        let token = self.get_token()?;

        let response = self
            .client
            .delete(format!(
                "{}/channels/{}/messages/{}",
                API_BASE, channel_id, message_id
            ))
            .header("Authorization", format!("Bot {}", token))
            .send()
            .await
            .context("Failed to delete message")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Discord API error: {} - {}", status, text);
        }

        Ok(())
    }

    /// Add a reaction to a message.
    pub async fn add_reaction(
        &self,
        channel_id: &str,
        message_id: &str,
        emoji: &str,
    ) -> Result<()> {
        let token = self.get_token()?;

        // URL encode the emoji
        let encoded_emoji = urlencoding::encode(emoji);

        let response = self
            .client
            .put(format!(
                "{}/channels/{}/messages/{}/reactions/{}/@me",
                API_BASE, channel_id, message_id, encoded_emoji
            ))
            .header("Authorization", format!("Bot {}", token))
            .send()
            .await
            .context("Failed to add reaction")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Discord API error: {} - {}", status, text);
        }

        Ok(())
    }

    /// Reply to a message.
    pub async fn reply_to_message(
        &self,
        channel_id: &str,
        message_id: &str,
        content: &str,
    ) -> Result<Message> {
        let token = self.get_token()?;

        let body = serde_json::json!({
            "content": content,
            "message_reference": {
                "message_id": message_id
            }
        });

        let response = self
            .client
            .post(format!("{}/channels/{}/messages", API_BASE, channel_id))
            .header("Authorization", format!("Bot {}", token))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Failed to reply to message")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Discord API error: {} - {}", status, text);
        }

        response.json().await.context("Failed to parse message")
    }

    /// Get a user by ID.
    pub async fn get_user(&self, user_id: &str) -> Result<User> {
        let token = self.get_token()?;

        let response = self
            .client
            .get(format!("{}/users/{}", API_BASE, user_id))
            .header("Authorization", format!("Bot {}", token))
            .send()
            .await
            .context("Failed to get user")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Discord API error: {} - {}", status, text);
        }

        response.json().await.context("Failed to parse user")
    }
}

// URL encoding helper
mod urlencoding {
    pub fn encode(s: &str) -> String {
        let mut result = String::new();
        for c in s.chars() {
            match c {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => result.push(c),
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

#[cfg(test)]
mod tests {
    use super::{urlencoding, DiscordClient};
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    struct EnvGuard {
        key: &'static str,
        previous: Option<String>,
    }

    impl EnvGuard {
        fn set(key: &'static str, value: &str) -> Self {
            let previous = std::env::var(key).ok();
            std::env::set_var(key, value);
            Self { key, previous }
        }

        fn remove(key: &'static str) -> Self {
            let previous = std::env::var(key).ok();
            std::env::remove_var(key);
            Self { key, previous }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            match &self.previous {
                Some(value) => std::env::set_var(self.key, value),
                None => std::env::remove_var(self.key),
            }
        }
    }

    #[test]
    fn encode_preserves_safe_chars() {
        let encoded = urlencoding::encode("Az09-_.~");
        assert_eq!(encoded, "Az09-_.~");
    }

    #[test]
    fn encode_escapes_spaces_and_unicode() {
        let encoded = urlencoding::encode("hello world 😀");
        assert_eq!(encoded, "hello%20world%20%F0%9F%98%80");
    }

    #[test]
    fn client_reads_token_from_env() {
        let _lock = ENV_LOCK.lock().expect("env lock");
        let _guard = EnvGuard::set("DISCORD_BOT_TOKEN", "token");

        let client = DiscordClient::new().expect("client");
        assert!(client.has_token());
    }

    #[test]
    fn client_has_no_token_when_env_missing() {
        let _lock = ENV_LOCK.lock().expect("env lock");
        let _guard = EnvGuard::remove("DISCORD_BOT_TOKEN");

        let client = DiscordClient::new().expect("client");
        assert!(!client.has_token());
    }
}
