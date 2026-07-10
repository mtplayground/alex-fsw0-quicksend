use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct SendRequest {
    #[serde(alias = "recipient", alias = "to")]
    pub recipient_email: String,
    pub subject: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct SendResponse {
    pub status: &'static str,
    pub message: &'static str,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: ErrorBody,
}

impl ErrorResponse {
    pub fn invalid_json() -> Self {
        Self {
            error: ErrorBody {
                code: "invalid_json",
                message: "Request body must be valid JSON with recipient_email, subject, and message.",
                fields: Vec::new(),
            },
        }
    }

    pub fn validation_failed(fields: Vec<FieldError>) -> Self {
        Self {
            error: ErrorBody {
                code: "validation_failed",
                message: "Request payload failed validation.",
                fields,
            },
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub code: &'static str,
    pub message: &'static str,
    pub fields: Vec<FieldError>,
}

#[derive(Debug, Serialize)]
pub struct FieldError {
    pub field: &'static str,
    pub message: &'static str,
}
