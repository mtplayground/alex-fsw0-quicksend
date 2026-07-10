use axum::{extract::State, routing::get, Json, Router};
use serde::Serialize;

use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
}

pub fn build_router(config: Config) -> Router {
    let state = AppState { config };

    Router::new()
        .route("/health", get(health_check))
        .with_state(state)
}

async fn health_check(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        email_proxy_configured: state.config.email_proxy_configured(),
        rate_limit_requests_per_window: state.config.rate_limit_requests_per_window,
        rate_limit_window_seconds: state.config.rate_limit_window_seconds,
    })
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    email_proxy_configured: bool,
    rate_limit_requests_per_window: u32,
    rate_limit_window_seconds: u64,
}
