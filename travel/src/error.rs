//! Error types for the travel daemon.
//!
//! # CHANGELOG (recent first, max 5 entries)
//! 01/14/2026 - Initial implementation (Claude)

use thiserror::Error;

/// Errors that can occur during travel operations.
#[derive(Error, Debug)]
pub enum TravelError {
    /// API returned an error response.
    #[error("API error: {message}")]
    Api {
        message: String,
        status_code: Option<u16>,
    },

    /// Rate limit exceeded.
    #[error("Rate limit exceeded (retry after {retry_after}s)")]
    RateLimit { retry_after: u32 },

    /// Validation error for input parameters.
    #[error("Validation error: {message}")]
    Validation {
        message: String,
        field: Option<String>,
    },

    /// Network/HTTP error.
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    /// Failed to parse response.
    #[error("Parse error: {message}")]
    Parse { message: String },

    /// Request timed out.
    #[error("Request timeout")]
    Timeout,

    /// Location not found in lookup table.
    #[error("Unknown location: {0}")]
    UnknownLocation(String),
}

impl TravelError {
    /// Create a validation error.
    pub fn validation(message: impl Into<String>, field: Option<&str>) -> Self {
        Self::Validation {
            message: message.into(),
            field: field.map(|s| s.to_string()),
        }
    }

    /// Create an API error.
    pub fn api(message: impl Into<String>, status_code: Option<u16>) -> Self {
        Self::Api {
            message: message.into(),
            status_code,
        }
    }

    /// Create a parse error.
    pub fn parse(message: impl Into<String>) -> Self {
        Self::Parse {
            message: message.into(),
        }
    }
}
