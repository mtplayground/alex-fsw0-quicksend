use std::fmt;

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{config::Config, models::SendRequest};

#[derive(Clone)]
pub struct EmailClient {
    endpoint: Option<EmailEndpoint>,
    http_client: reqwest::Client,
}

impl EmailClient {
    pub fn from_config(config: &Config) -> Self {
        let endpoint = match (&config.email_proxy_url, &config.email_app_token) {
            (Some(url), Some(token)) => Some(EmailEndpoint {
                url: url.clone(),
                token: token.clone(),
            }),
            _ => None,
        };

        Self {
            endpoint,
            http_client: reqwest::Client::new(),
        }
    }

    pub async fn send(&self, request: &SendRequest) -> Result<EmailDeliveryOutcome, EmailError> {
        let Some(endpoint) = &self.endpoint else {
            return Ok(EmailDeliveryOutcome::SkippedNotConfigured);
        };

        let response = self
            .http_client
            .post(&endpoint.url)
            .bearer_auth(&endpoint.token)
            .json(&EmailProxyRequest {
                to: &request.recipient_email,
                subject: &request.subject,
                text: &request.message,
            })
            .send()
            .await
            .map_err(EmailError::Request)?;

        let status = response.status();
        if status == StatusCode::TOO_MANY_REQUESTS {
            return Err(EmailError::RateLimited);
        }

        if !status.is_success() {
            let body = response.text().await.map_or_else(
                |error| format!("unable to read email proxy error body: {error}"),
                truncate_error_body,
            );

            return Err(EmailError::Proxy {
                status: status.as_u16(),
                body,
            });
        }

        let proxy_response = response
            .json::<EmailProxyResponse>()
            .await
            .map_err(EmailError::InvalidResponse)?;

        Ok(EmailDeliveryOutcome::Delivered {
            message_id: proxy_response.id,
        })
    }
}

#[derive(Clone)]
struct EmailEndpoint {
    url: String,
    token: String,
}

#[derive(Serialize)]
struct EmailProxyRequest<'a> {
    to: &'a str,
    subject: &'a str,
    text: &'a str,
}

#[derive(Deserialize)]
struct EmailProxyResponse {
    id: Option<String>,
}

pub enum EmailDeliveryOutcome {
    Delivered { message_id: Option<String> },
    SkippedNotConfigured,
}

impl EmailDeliveryOutcome {
    pub fn status(&self) -> &'static str {
        match self {
            Self::Delivered { .. } => "sent",
            Self::SkippedNotConfigured => "skipped",
        }
    }

    pub fn response_message(&self) -> &'static str {
        match self {
            Self::Delivered { .. } => "Message delivered.",
            Self::SkippedNotConfigured => {
                "Message accepted; email delivery is not configured for this environment."
            }
        }
    }

    pub fn message_id(self) -> Option<String> {
        match self {
            Self::Delivered { message_id } => message_id,
            Self::SkippedNotConfigured => None,
        }
    }
}

#[derive(Debug)]
pub enum EmailError {
    RateLimited,
    Proxy { status: u16, body: String },
    Request(reqwest::Error),
    InvalidResponse(reqwest::Error),
}

impl fmt::Display for EmailError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RateLimited => write!(formatter, "email proxy rate limited the request"),
            Self::Proxy { status, body } => {
                write!(formatter, "email proxy returned status {status}: {body}")
            }
            Self::Request(error) => write!(formatter, "email proxy request failed: {error}"),
            Self::InvalidResponse(error) => {
                write!(formatter, "email proxy returned an invalid response: {error}")
            }
        }
    }
}

impl std::error::Error for EmailError {}

fn truncate_error_body(body: String) -> String {
    const MAX_ERROR_BODY_LENGTH: usize = 512;

    body.chars().take(MAX_ERROR_BODY_LENGTH).collect()
}
