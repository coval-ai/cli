use reqwest::StatusCode;
use thiserror::Error;

use super::models::ErrorResponse;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Authentication failed: {message}")]
    Unauthenticated { message: String },

    #[error("{resource} not found")]
    NotFound { resource: String },

    #[error("Invalid request: {message}")]
    InvalidArgument {
        message: String,
        field: Option<String>,
    },

    #[error("Permission denied: {message}")]
    PermissionDenied { message: String },

    #[error("Server error: {message}")]
    Internal { message: String },

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
}

impl ApiError {
    pub fn from_response(_status: StatusCode, resp: ErrorResponse) -> Self {
        let field = resp.error.details.first().and_then(|d| d.field.clone());

        // Append detail descriptions to the message so users see what's actually wrong
        let detail_descriptions: Vec<String> = resp
            .error
            .details
            .iter()
            .filter_map(|d| d.description.as_ref())
            .map(|s| s.to_string())
            .collect();

        let message = if detail_descriptions.is_empty() {
            resp.error.message
        } else {
            format!("{}: {}", resp.error.message, detail_descriptions.join("; "))
        };

        match resp.error.code.as_str() {
            "UNAUTHENTICATED" => Self::Unauthenticated { message },
            "NOT_FOUND" => Self::NotFound { resource: message },
            "INVALID_ARGUMENT" => Self::InvalidArgument { message, field },
            "PERMISSION_DENIED" => Self::PermissionDenied { message },
            _ => Self::Internal { message },
        }
    }

    #[allow(dead_code)]
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Unauthenticated { .. } | Self::PermissionDenied { .. } => 2,
            Self::NotFound { .. } => 3,
            Self::InvalidArgument { .. } => 4,
            Self::Network(_) => 5,
            Self::Internal { .. } => 1,
        }
    }
}
