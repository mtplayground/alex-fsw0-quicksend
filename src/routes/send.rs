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
