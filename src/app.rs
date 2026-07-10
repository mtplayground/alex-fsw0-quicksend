use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;
use tower_http::services::{ServeDir, ServeFile};

use crate::config::Config;
use crate::routes;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
}

pub fn build_router(config: Config) -> Router {
    let frontend_dist_dir = config.frontend_dist_dir.clone();
    let index_path = format!("{frontend_dist_dir}/index.html");
    let static_files = ServeDir::new(frontend_dist_dir).fallback(ServeFile::new(index_path));
    let state = AppState { config };

    Router::new()
        .route("/health", get(health_check))
        .route("/api/send", post(routes::send::send))
        .fallback_service(static_files)
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
