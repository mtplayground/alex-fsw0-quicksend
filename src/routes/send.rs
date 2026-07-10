use axum::{
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

use crate::{
    models::{ErrorResponse, SendRequest, SendResponse},
    validation::validate_send_request,
};

pub async fn send(payload: Result<Json<SendRequest>, JsonRejection>) -> Response {
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

    (
        StatusCode::ACCEPTED,
        Json(SendResponse {
            status: "accepted",
            message: "Message payload accepted.",
        }),
    )
        .into_response()
}
