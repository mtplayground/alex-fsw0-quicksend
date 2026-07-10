use axum::{
    extract::State,
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

use crate::{
    app::AppState,
    email::EmailError,
    models::{ErrorResponse, SendRequest, SendResponse},
    validation::validate_send_request,
};

pub async fn send(
    State(state): State<AppState>,
    payload: Result<Json<SendRequest>, JsonRejection>,
) -> Response {
    let Json(request) = match payload {
        Ok(payload) => payload,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, Json(ErrorResponse::invalid_json())).into_response();
        }
    };

    if let Err(fields) = validate_send_request(&request) {
        return (StatusCode::BAD_REQUEST, Json(ErrorResponse::validation_failed(fields)))
            .into_response();
    }

    let delivery_outcome = match state.email_client.send(&request).await {
        Ok(outcome) => outcome,
        Err(EmailError::RateLimited) => {
            return (
                StatusCode::TOO_MANY_REQUESTS,
                Json(ErrorResponse::email_rate_limited()),
            )
                .into_response();
        }
        Err(error) => {
            eprintln!("email delivery failed: {error}");
            return (
                StatusCode::BAD_GATEWAY,
                Json(ErrorResponse::delivery_failed()),
            )
                .into_response();
        }
    };

    (
        StatusCode::ACCEPTED,
        Json(SendResponse {
            status: "accepted",
            message: delivery_outcome.response_message(),
            delivery_status: delivery_outcome.status(),
            message_id: delivery_outcome.message_id(),
        }),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use std::{net::IpAddr, time::Duration};

    use axum::{body::to_bytes, extract::State, http::StatusCode, Json};

    use super::send;
    use crate::{
        app::AppState,
        config::Config,
        email::{EmailClient, EmailDeliveryOutcome, EmailError},
        models::SendRequest,
        rate_limit::RateLimiter,
    };

    #[tokio::test]
    async fn sends_with_mocked_email_success() {
        let state = app_state(EmailClient::mock(vec![Ok(EmailDeliveryOutcome::Delivered {
            message_id: Some("msg_123".to_string()),
        })]));

        let response = send(State(state), Ok(Json(valid_request()))).await;
        let status = response.status();
        let body = response_body(response).await;

        assert_eq!(status, StatusCode::ACCEPTED);
        assert!(body.contains("\"delivery_status\":\"sent\""));
        assert!(body.contains("\"message_id\":\"msg_123\""));
    }

    #[tokio::test]
    async fn maps_mocked_email_failure_to_bad_gateway() {
        let state = app_state(EmailClient::mock(vec![Err(EmailError::Proxy {
            status: 500,
            body: "proxy unavailable".to_string(),
        })]));

        let response = send(State(state), Ok(Json(valid_request()))).await;
        let status = response.status();
        let body = response_body(response).await;

        assert_eq!(status, StatusCode::BAD_GATEWAY);
        assert!(body.contains("\"code\":\"delivery_failed\""));
    }

    fn app_state(email_client: EmailClient) -> AppState {
        let config = test_config();
        AppState {
            config,
            email_client,
            rate_limiter: RateLimiter::new(10, Duration::from_secs(60)),
        }
    }

    fn test_config() -> Config {
        Config {
            host: IpAddr::from([0, 0, 0, 0]),
            port: 8080,
            frontend_dist_dir: "frontend/dist".to_string(),
            email_proxy_url: None,
            email_app_token: None,
            rate_limit_requests_per_window: 10,
            rate_limit_window_seconds: 60,
        }
    }

    fn valid_request() -> SendRequest {
        SendRequest {
            recipient_email: "person@example.com".to_string(),
            subject: "Hello".to_string(),
            message: "A short message".to_string(),
        }
    }

    async fn response_body(response: axum::response::Response) -> String {
        let bytes = match to_bytes(response.into_body(), usize::MAX).await {
            Ok(bytes) => bytes,
            Err(error) => panic!("failed to read response body: {error}"),
        };

        match String::from_utf8(bytes.to_vec()) {
            Ok(body) => body,
            Err(error) => panic!("response body was not valid utf-8: {error}"),
        }
    }
}
