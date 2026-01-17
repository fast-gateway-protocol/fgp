//! Data models for Google Docs API.
//!
//! # CHANGELOG (recent first, max 5 entries)
//! 01/15/2026 - Initial implementation (Claude)

use serde::{Deserialize, Serialize};

/// A Google Doc document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    #[serde(rename = "documentId")]
    pub document_id: String,
    pub title: String,
    #[serde(default)]
    pub body: Option<Body>,
    #[serde(default, rename = "revisionId")]
    pub revision_id: Option<String>,
}

/// Document body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Body {
    #[serde(default)]
    pub content: Option<Vec<StructuralElement>>,
}

/// A structural element in the document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuralElement {
    #[serde(default, rename = "startIndex")]
    pub start_index: Option<i64>,
    #[serde(default, rename = "endIndex")]
    pub end_index: Option<i64>,
    #[serde(default)]
    pub paragraph: Option<Paragraph>,
    #[serde(default)]
    pub table: Option<serde_json::Value>,
    #[serde(default, rename = "sectionBreak")]
    pub section_break: Option<serde_json::Value>,
}

/// A paragraph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paragraph {
    #[serde(default)]
    pub elements: Option<Vec<ParagraphElement>>,
    #[serde(default, rename = "paragraphStyle")]
    pub paragraph_style: Option<serde_json::Value>,
}

/// An element within a paragraph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParagraphElement {
    #[serde(default, rename = "startIndex")]
    pub start_index: Option<i64>,
    #[serde(default, rename = "endIndex")]
    pub end_index: Option<i64>,
    #[serde(default, rename = "textRun")]
    pub text_run: Option<TextRun>,
}

/// A run of text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextRun {
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default, rename = "textStyle")]
    pub text_style: Option<serde_json::Value>,
}

/// Response from batch update.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchUpdateResponse {
    #[serde(rename = "documentId")]
    pub document_id: String,
    #[serde(default)]
    pub replies: Option<Vec<serde_json::Value>>,
    #[serde(default, rename = "writeControl")]
    pub write_control: Option<serde_json::Value>,
}

/// Extracted plain text from document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlainTextExtract {
    pub document_id: String,
    pub title: String,
    pub text: String,
    pub char_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn document_defaults_body_and_revision() {
        let value = json!({
            "documentId": "doc_1",
            "title": "Doc"
        });
        let doc: Document = serde_json::from_value(value).expect("deserialize");

        assert_eq!(doc.document_id, "doc_1");
        assert!(doc.body.is_none());
        assert!(doc.revision_id.is_none());
    }

    #[test]
    fn paragraph_text_run_maps_content() {
        let value = json!({
            "elements": [
                {
                    "startIndex": 1,
                    "endIndex": 5,
                    "textRun": {
                        "content": "Hi"
                    }
                }
            ]
        });
        let paragraph: Paragraph = serde_json::from_value(value).expect("deserialize");
        let element = paragraph.elements.expect("elements")[0].clone();
        let text_run = element.text_run.expect("text run");

        assert_eq!(text_run.content.as_deref(), Some("Hi"));
        assert_eq!(element.start_index, Some(1));
        assert_eq!(element.end_index, Some(5));
    }

    #[test]
    fn batch_update_response_optional_fields() {
        let value = json!({
            "documentId": "doc_1"
        });
        let response: BatchUpdateResponse = serde_json::from_value(value).expect("deserialize");

        assert_eq!(response.document_id, "doc_1");
        assert!(response.replies.is_none());
        assert!(response.write_control.is_none());
    }
}
