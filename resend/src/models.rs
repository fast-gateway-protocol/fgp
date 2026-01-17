//! Data models for Resend API requests and responses.

use serde::{Deserialize, Serialize};

/// Request to send a single email.
#[derive(Debug, Clone, Serialize)]
pub struct SendEmailRequest {
    pub from: String,
    pub to: Vec<String>,
    pub subject: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cc: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bcc: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<Tag>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<Attachment>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<std::collections::HashMap<String, String>>,
}

/// Email tag for categorization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub name: String,
    pub value: String,
}

/// Email attachment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    /// Base64-encoded content or a URL
    pub content: String,
    pub filename: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
}

/// Response from sending an email.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SendEmailResponse {
    pub id: String,
}

/// Request to send batch emails.
#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
pub struct BatchEmailRequest {
    pub emails: Vec<SendEmailRequest>,
}

/// Response from sending batch emails.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BatchEmailResponse {
    pub data: Vec<SendEmailResponse>,
}

/// Email details returned from GET /emails/{id}.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Email {
    pub id: String,
    pub from: String,
    pub to: Vec<String>,
    pub subject: String,
    #[serde(default)]
    pub cc: Vec<String>,
    #[serde(default)]
    pub bcc: Vec<String>,
    #[serde(default)]
    pub reply_to: Option<String>,
    pub created_at: String,
    #[serde(default)]
    pub last_event: Option<String>,
    #[serde(default)]
    pub html: Option<String>,
    #[serde(default)]
    pub text: Option<String>,
}

/// Verified domain.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Domain {
    pub id: String,
    pub name: String,
    pub status: String,
    pub created_at: String,
    pub region: String,
    #[serde(default)]
    pub records: Vec<DnsRecord>,
}

/// DNS record for domain verification.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DnsRecord {
    pub record: String,
    pub name: String,
    pub value: String,
    #[serde(rename = "type")]
    pub record_type: String,
    #[serde(default)]
    pub ttl: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub priority: Option<i32>,
}

/// Response from listing domains.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DomainsResponse {
    pub data: Vec<Domain>,
}

/// Resend API error response.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct ApiError {
    #[serde(default, rename = "statusCode")]
    pub status_code: Option<i32>,
    pub message: String,
    #[serde(default)]
    pub name: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_send_email_request_serialization() {
        let request = SendEmailRequest {
            from: "sender@example.com".to_string(),
            to: vec!["recipient@example.com".to_string()],
            subject: "Test Subject".to_string(),
            html: Some("<p>Hello</p>".to_string()),
            text: None,
            cc: None,
            bcc: None,
            reply_to: None,
            tags: None,
            attachments: None,
            headers: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("sender@example.com"));
        assert!(json.contains("recipient@example.com"));
        assert!(json.contains("Test Subject"));
        // text should be omitted since it's None
        assert!(!json.contains("\"text\""));
    }

    #[test]
    fn test_send_email_response_deserialization() {
        let json = r#"{"id": "email_123abc"}"#;
        let response: SendEmailResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.id, "email_123abc");
    }

    #[test]
    fn test_email_deserialization() {
        let json = r#"{
            "id": "email_123",
            "from": "sender@example.com",
            "to": ["recipient@example.com"],
            "subject": "Hello",
            "created_at": "2024-01-14T00:00:00Z"
        }"#;
        let email: Email = serde_json::from_str(json).unwrap();
        assert_eq!(email.id, "email_123");
        assert_eq!(email.to.len(), 1);
    }

    #[test]
    fn test_domain_deserialization() {
        let json = r#"{
            "id": "domain_123",
            "name": "example.com",
            "status": "verified",
            "created_at": "2024-01-14T00:00:00Z",
            "region": "us-east-1",
            "records": []
        }"#;
        let domain: Domain = serde_json::from_str(json).unwrap();
        assert_eq!(domain.name, "example.com");
        assert_eq!(domain.status, "verified");
    }

    #[test]
    fn test_batch_email_response_deserialization() {
        let json = r#"{"data": [{"id": "email_1"}, {"id": "email_2"}]}"#;
        let response: BatchEmailResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.data.len(), 2);
        assert_eq!(response.data[0].id, "email_1");
    }
}
