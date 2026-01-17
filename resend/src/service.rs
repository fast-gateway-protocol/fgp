//! FGP service implementation for Resend.

use anyhow::Result;
use fgp_daemon::schema::SchemaBuilder;
use fgp_daemon::service::{HealthStatus, MethodInfo};
use fgp_daemon::FgpService;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Runtime;

use crate::api::ResendClient;
use crate::models::{Attachment, SendEmailRequest, Tag};

/// FGP service for Resend email operations.
pub struct ResendService {
    client: Arc<ResendClient>,
    runtime: Runtime,
}

impl ResendService {
    /// Create a new ResendService.
    ///
    /// API key is resolved from RESEND_API_KEY environment variable.
    pub fn new(api_key: Option<String>) -> Result<Self> {
        let client = ResendClient::new(api_key)?;
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

    /// Helper to get a string array parameter.
    fn get_str_array(params: &HashMap<String, Value>, key: &str) -> Option<Vec<String>> {
        params.get(key).and_then(|v| {
            v.as_array().map(|arr| {
                arr.iter()
                    .filter_map(|item| item.as_str().map(|s| s.to_string()))
                    .collect()
            })
        })
    }

    // ========================================================================
    // Method implementations
    // ========================================================================

    fn health(&self) -> Result<Value> {
        let client = self.client.clone();
        let ok = self.runtime.block_on(async move { client.ping().await })?;

        Ok(json!({
            "status": if ok { "healthy" } else { "unhealthy" },
            "api_connected": ok,
            "version": env!("CARGO_PKG_VERSION"),
        }))
    }

    fn send_email(&self, params: HashMap<String, Value>) -> Result<Value> {
        let from = Self::get_str(&params, "from")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: from"))?
            .to_string();

        let to = Self::get_str_array(&params, "to")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: to"))?;

        if to.is_empty() {
            anyhow::bail!("'to' must contain at least one recipient");
        }

        let subject = Self::get_str(&params, "subject")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: subject"))?
            .to_string();

        let html = Self::get_str(&params, "html").map(|s| s.to_string());
        let text = Self::get_str(&params, "text").map(|s| s.to_string());

        if html.is_none() && text.is_none() {
            anyhow::bail!("Either 'html' or 'text' content is required");
        }

        let cc = Self::get_str_array(&params, "cc");
        let bcc = Self::get_str_array(&params, "bcc");
        let reply_to = Self::get_str(&params, "reply_to").map(|s| s.to_string());

        // Parse tags if provided
        let tags: Option<Vec<Tag>> = params.get("tags").and_then(|v| {
            v.as_array().map(|arr| {
                arr.iter()
                    .filter_map(|item| {
                        let name = item.get("name")?.as_str()?;
                        let value = item.get("value")?.as_str()?;
                        Some(Tag {
                            name: name.to_string(),
                            value: value.to_string(),
                        })
                    })
                    .collect()
            })
        });

        // Parse attachments if provided
        let attachments: Option<Vec<Attachment>> = params.get("attachments").and_then(|v| {
            v.as_array().map(|arr| {
                arr.iter()
                    .filter_map(|item| {
                        let content = item.get("content")?.as_str()?;
                        let filename = item.get("filename")?.as_str()?;
                        let content_type = item
                            .get("content_type")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        Some(Attachment {
                            content: content.to_string(),
                            filename: filename.to_string(),
                            content_type,
                        })
                    })
                    .collect()
            })
        });

        // Parse headers if provided
        let headers: Option<HashMap<String, String>> = params.get("headers").and_then(|v| {
            v.as_object().map(|obj| {
                obj.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            })
        });

        let request = SendEmailRequest {
            from,
            to,
            subject,
            html,
            text,
            cc,
            bcc,
            reply_to,
            tags,
            attachments,
            headers,
        };

        let client = self.client.clone();
        let response = self
            .runtime
            .block_on(async move { client.send_email(request).await })?;

        Ok(json!({
            "sent": true,
            "id": response.id,
        }))
    }

    fn send_batch(&self, params: HashMap<String, Value>) -> Result<Value> {
        let emails_value = params
            .get("emails")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: emails"))?;

        let emails_array = emails_value
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("'emails' must be an array"))?;

        if emails_array.is_empty() {
            anyhow::bail!("'emails' must contain at least one email");
        }

        if emails_array.len() > 100 {
            anyhow::bail!("Batch cannot exceed 100 emails");
        }

        let mut requests = Vec::new();

        for (i, email) in emails_array.iter().enumerate() {
            let from = email
                .get("from")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Email {} missing 'from'", i))?
                .to_string();

            let to: Vec<String> = email
                .get("to")
                .and_then(|v| {
                    v.as_array().map(|arr| {
                        arr.iter()
                            .filter_map(|item| item.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                })
                .ok_or_else(|| anyhow::anyhow!("Email {} missing 'to'", i))?;

            let subject = email
                .get("subject")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Email {} missing 'subject'", i))?
                .to_string();

            let html = email.get("html").and_then(|v| v.as_str()).map(|s| s.to_string());
            let text = email.get("text").and_then(|v| v.as_str()).map(|s| s.to_string());

            if html.is_none() && text.is_none() {
                anyhow::bail!("Email {} missing 'html' or 'text' content", i);
            }

            let cc = email.get("cc").and_then(|v| {
                v.as_array().map(|arr| {
                    arr.iter()
                        .filter_map(|item| item.as_str().map(|s| s.to_string()))
                        .collect()
                })
            });

            let bcc = email.get("bcc").and_then(|v| {
                v.as_array().map(|arr| {
                    arr.iter()
                        .filter_map(|item| item.as_str().map(|s| s.to_string()))
                        .collect()
                })
            });

            let reply_to = email.get("reply_to").and_then(|v| v.as_str()).map(|s| s.to_string());

            requests.push(SendEmailRequest {
                from,
                to,
                subject,
                html,
                text,
                cc,
                bcc,
                reply_to,
                tags: None,
                attachments: None,
                headers: None,
            });
        }

        let client = self.client.clone();
        let response = self
            .runtime
            .block_on(async move { client.send_batch(requests).await })?;

        let ids: Vec<&str> = response.data.iter().map(|r| r.id.as_str()).collect();

        Ok(json!({
            "sent": true,
            "count": ids.len(),
            "ids": ids,
        }))
    }

    fn get_email(&self, params: HashMap<String, Value>) -> Result<Value> {
        let email_id = Self::get_str(&params, "id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: id"))?;

        let client = self.client.clone();
        let email_id = email_id.to_string();
        let email = self
            .runtime
            .block_on(async move { client.get_email(&email_id).await })?;

        Ok(serde_json::to_value(email)?)
    }

    fn list_domains(&self) -> Result<Value> {
        let client = self.client.clone();
        let domains = self
            .runtime
            .block_on(async move { client.list_domains().await })?;

        Ok(json!({
            "domains": domains,
            "count": domains.len(),
        }))
    }
}

impl FgpService for ResendService {
    fn name(&self) -> &str {
        "resend"
    }

    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    fn dispatch(&self, method: &str, params: HashMap<String, Value>) -> Result<Value> {
        match method {
            "health" => self.health(),
            "send" | "resend.send" => self.send_email(params),
            "batch" | "resend.batch" => self.send_batch(params),
            "get" | "resend.get" => self.get_email(params),
            "domains" | "resend.domains" => self.list_domains(),
            _ => anyhow::bail!("Unknown method: {}", method),
        }
    }

    fn method_list(&self) -> Vec<MethodInfo> {
        vec![
            // resend.send - Send a single email
            MethodInfo::new("resend.send", "Send a single email")
                .schema(
                    SchemaBuilder::object()
                        .property(
                            "from",
                            SchemaBuilder::string()
                                .format("email")
                                .description("Sender email address (must be from a verified domain)"),
                        )
                        .property(
                            "to",
                            SchemaBuilder::array()
                                .items(SchemaBuilder::string().format("email"))
                                .min_items(1)
                                .description("Recipient email addresses"),
                        )
                        .property(
                            "subject",
                            SchemaBuilder::string()
                                .min_length(1)
                                .description("Email subject line"),
                        )
                        .property(
                            "html",
                            SchemaBuilder::string().description("HTML content of the email"),
                        )
                        .property(
                            "text",
                            SchemaBuilder::string().description("Plain text content of the email"),
                        )
                        .property(
                            "cc",
                            SchemaBuilder::array()
                                .items(SchemaBuilder::string().format("email"))
                                .description("CC recipients"),
                        )
                        .property(
                            "bcc",
                            SchemaBuilder::array()
                                .items(SchemaBuilder::string().format("email"))
                                .description("BCC recipients"),
                        )
                        .property(
                            "reply_to",
                            SchemaBuilder::string()
                                .format("email")
                                .description("Reply-to email address"),
                        )
                        .property(
                            "tags",
                            SchemaBuilder::array()
                                .items(
                                    SchemaBuilder::object()
                                        .property("name", SchemaBuilder::string())
                                        .property("value", SchemaBuilder::string()),
                                )
                                .description("Email tags for categorization"),
                        )
                        .property(
                            "attachments",
                            SchemaBuilder::array()
                                .items(
                                    SchemaBuilder::object()
                                        .property("content", SchemaBuilder::string().description("Base64-encoded content"))
                                        .property("filename", SchemaBuilder::string())
                                        .property("content_type", SchemaBuilder::string()),
                                )
                                .description("File attachments"),
                        )
                        .required(&["from", "to", "subject"])
                        .build(),
                )
                .returns(
                    SchemaBuilder::object()
                        .property("sent", SchemaBuilder::boolean())
                        .property("id", SchemaBuilder::string().description("Email ID"))
                        .build(),
                )
                .example(
                    "Send a text email",
                    json!({
                        "from": "hello@example.com",
                        "to": ["user@recipient.com"],
                        "subject": "Hello World",
                        "text": "This is a test email."
                    }),
                )
                .example(
                    "Send an HTML email",
                    json!({
                        "from": "hello@example.com",
                        "to": ["user@recipient.com"],
                        "subject": "Welcome!",
                        "html": "<h1>Welcome</h1><p>Thanks for signing up.</p>"
                    }),
                )
                .errors(&["INVALID_FROM", "DOMAIN_NOT_VERIFIED", "RATE_LIMIT"]),

            // resend.batch - Send batch emails
            MethodInfo::new("resend.batch", "Send multiple emails in a single request (up to 100)")
                .schema(
                    SchemaBuilder::object()
                        .property(
                            "emails",
                            SchemaBuilder::array()
                                .items(
                                    SchemaBuilder::object()
                                        .property("from", SchemaBuilder::string().format("email"))
                                        .property("to", SchemaBuilder::array().items(SchemaBuilder::string().format("email")))
                                        .property("subject", SchemaBuilder::string())
                                        .property("html", SchemaBuilder::string())
                                        .property("text", SchemaBuilder::string())
                                        .property("cc", SchemaBuilder::array().items(SchemaBuilder::string()))
                                        .property("bcc", SchemaBuilder::array().items(SchemaBuilder::string()))
                                        .property("reply_to", SchemaBuilder::string()),
                                )
                                .min_items(1)
                                .max_items(100)
                                .description("Array of emails to send"),
                        )
                        .required(&["emails"])
                        .build(),
                )
                .returns(
                    SchemaBuilder::object()
                        .property("sent", SchemaBuilder::boolean())
                        .property("count", SchemaBuilder::integer())
                        .property("ids", SchemaBuilder::array().items(SchemaBuilder::string()))
                        .build(),
                )
                .example(
                    "Send batch emails",
                    json!({
                        "emails": [
                            {"from": "hello@example.com", "to": ["user1@test.com"], "subject": "Hi", "text": "Hello 1"},
                            {"from": "hello@example.com", "to": ["user2@test.com"], "subject": "Hi", "text": "Hello 2"}
                        ]
                    }),
                )
                .errors(&["INVALID_FROM", "BATCH_LIMIT_EXCEEDED", "RATE_LIMIT"]),

            // resend.get - Get email by ID
            MethodInfo::new("resend.get", "Get email details by ID")
                .schema(
                    SchemaBuilder::object()
                        .property(
                            "id",
                            SchemaBuilder::string().description("Email ID returned from send"),
                        )
                        .required(&["id"])
                        .build(),
                )
                .returns(
                    SchemaBuilder::object()
                        .property("id", SchemaBuilder::string())
                        .property("from", SchemaBuilder::string())
                        .property("to", SchemaBuilder::array().items(SchemaBuilder::string()))
                        .property("subject", SchemaBuilder::string())
                        .property("created_at", SchemaBuilder::string().format("date-time"))
                        .property("last_event", SchemaBuilder::string().description("Last delivery event"))
                        .build(),
                )
                .example("Get email details", json!({"id": "email_123abc"}))
                .errors(&["NOT_FOUND"]),

            // resend.domains - List verified domains
            MethodInfo::new("resend.domains", "List all verified domains")
                .schema(SchemaBuilder::object().build())
                .returns(
                    SchemaBuilder::object()
                        .property(
                            "domains",
                            SchemaBuilder::array().items(
                                SchemaBuilder::object()
                                    .property("id", SchemaBuilder::string())
                                    .property("name", SchemaBuilder::string())
                                    .property("status", SchemaBuilder::string())
                                    .property("region", SchemaBuilder::string())
                                    .property("created_at", SchemaBuilder::string().format("date-time")),
                            ),
                        )
                        .property("count", SchemaBuilder::integer())
                        .build(),
                )
                .example("List domains", json!({})),

            // resend.health - Check API connectivity
            MethodInfo::new("resend.health", "Check Resend API connectivity")
                .schema(SchemaBuilder::object().build())
                .returns(
                    SchemaBuilder::object()
                        .property("status", SchemaBuilder::string())
                        .property("api_connected", SchemaBuilder::boolean())
                        .property("version", SchemaBuilder::string())
                        .build(),
                )
                .example("Health check", json!({})),
        ]
    }

    fn on_start(&self) -> Result<()> {
        tracing::info!("ResendService starting, verifying API connection...");
        let client = self.client.clone();
        self.runtime.block_on(async move {
            match client.ping().await {
                Ok(true) => {
                    tracing::info!("Resend API connection verified");
                    Ok(())
                }
                Ok(false) => {
                    tracing::warn!("Resend API health check returned false");
                    Ok(())
                }
                Err(e) => {
                    tracing::error!("Failed to connect to Resend API: {}", e);
                    Err(e)
                }
            }
        })
    }

    fn health_check(&self) -> HashMap<String, HealthStatus> {
        let mut checks = HashMap::new();

        let client = self.client.clone();
        let start = std::time::Instant::now();
        let result = self.runtime.block_on(async move { client.ping().await });

        let latency = start.elapsed().as_secs_f64() * 1000.0;

        match result {
            Ok(true) => {
                checks.insert(
                    "resend_api".into(),
                    HealthStatus::healthy_with_latency(latency),
                );
            }
            Ok(false) => {
                checks.insert(
                    "resend_api".into(),
                    HealthStatus::unhealthy("API health check failed"),
                );
            }
            Err(e) => {
                checks.insert("resend_api".into(), HealthStatus::unhealthy(e.to_string()));
            }
        }

        checks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_str_returns_string_value() {
        let mut params = HashMap::new();
        params.insert("from".to_string(), json!("test@example.com"));
        assert_eq!(ResendService::get_str(&params, "from"), Some("test@example.com"));
    }

    #[test]
    fn get_str_returns_none_for_non_string() {
        let mut params = HashMap::new();
        params.insert("from".to_string(), json!(42));
        assert!(ResendService::get_str(&params, "from").is_none());
    }

    #[test]
    fn get_str_array_returns_array() {
        let mut params = HashMap::new();
        params.insert("to".to_string(), json!(["a@test.com", "b@test.com"]));
        let result = ResendService::get_str_array(&params, "to").unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "a@test.com");
    }

    #[test]
    fn get_str_array_returns_none_for_non_array() {
        let mut params = HashMap::new();
        params.insert("to".to_string(), json!("not an array"));
        assert!(ResendService::get_str_array(&params, "to").is_none());
    }

    #[test]
    fn dispatch_requires_from_for_send() {
        let service = ResendService::new(Some("re_test".into())).unwrap();
        let err = service
            .dispatch("resend.send", HashMap::new())
            .expect_err("expected missing from error");
        assert!(err.to_string().contains("from"));
    }

    #[test]
    fn dispatch_requires_to_for_send() {
        let service = ResendService::new(Some("re_test".into())).unwrap();
        let mut params = HashMap::new();
        params.insert("from".to_string(), json!("test@example.com"));
        let err = service
            .dispatch("resend.send", params)
            .expect_err("expected missing to error");
        assert!(err.to_string().contains("to"));
    }

    #[test]
    fn dispatch_requires_subject_for_send() {
        let service = ResendService::new(Some("re_test".into())).unwrap();
        let mut params = HashMap::new();
        params.insert("from".to_string(), json!("test@example.com"));
        params.insert("to".to_string(), json!(["recipient@test.com"]));
        let err = service
            .dispatch("resend.send", params)
            .expect_err("expected missing subject error");
        assert!(err.to_string().contains("subject"));
    }

    #[test]
    fn dispatch_requires_content_for_send() {
        let service = ResendService::new(Some("re_test".into())).unwrap();
        let mut params = HashMap::new();
        params.insert("from".to_string(), json!("test@example.com"));
        params.insert("to".to_string(), json!(["recipient@test.com"]));
        params.insert("subject".to_string(), json!("Test"));
        let err = service
            .dispatch("resend.send", params)
            .expect_err("expected missing content error");
        assert!(err.to_string().contains("html") || err.to_string().contains("text"));
    }

    #[test]
    fn dispatch_rejects_unknown_method() {
        let service = ResendService::new(Some("re_test".into())).unwrap();
        let err = service
            .dispatch("resend.nope", HashMap::new())
            .expect_err("expected unknown method error");
        assert!(err.to_string().contains("Unknown method"));
    }
}
