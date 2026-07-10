use axum::{
    extract::State,
    middleware,
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;
use tower_http::services::{ServeDir, ServeFile};

use crate::config::Config;
use crate::email::EmailClient;
use crate::rate_limit::{self, RateLimiter};
use crate::routes;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub email_client: EmailClient,
    pub rate_limiter: RateLimiter,
}

pub fn build_router(config: Config) -> Router {
    let frontend_dist_dir = config.frontend_dist_dir.clone();
    let index_path = format!("{frontend_dist_dir}/index.html");
    let static_files = ServeDir::new(frontend_dist_dir).fallback(ServeFile::new(index_path));
    let email_client = EmailClient::from_config(&config);
    let rate_limiter = RateLimiter::from_config(&config);
    let state = AppState {
        config,
        email_client,
        rate_limiter,
    };
    let send_route = post(routes::send::send).route_layer(middleware::from_fn_with_state(
        state.clone(),
        rate_limit::enforce_send_rate_limit,
    ));

    Router::new()
        .route("/health", get(health_check))
        .route("/api/send", send_route)
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
