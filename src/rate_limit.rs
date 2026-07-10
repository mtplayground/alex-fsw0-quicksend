use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use axum::{
    body::Body,
    extract::State,
    http::{header, HeaderMap, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};

use crate::{app::AppState, config::Config, models::ErrorResponse};

#[derive(Clone)]
pub struct RateLimiter {
    buckets: Arc<Mutex<HashMap<String, RateLimitBucket>>>,
    limit: u32,
    window: Duration,
}

impl RateLimiter {
    pub fn from_config(config: &Config) -> Self {
        Self::new(
            config.rate_limit_requests_per_window,
            Duration::from_secs(config.rate_limit_window_seconds),
        )
    }

    pub fn new(limit: u32, window: Duration) -> Self {
        Self {
            buckets: Arc::new(Mutex::new(HashMap::new())),
            limit,
            window,
        }
    }

    pub fn check(&self, key: &str) -> RateLimitDecision {
        let now = Instant::now();
        let Ok(mut buckets) = self.buckets.lock() else {
            eprintln!("rate limiter state lock is poisoned; allowing request");
            return RateLimitDecision::Allowed;
        };

        buckets.retain(|_, bucket| now.duration_since(bucket.window_started_at) < self.window);

        let bucket = buckets
            .entry(key.to_string())
            .or_insert_with(|| RateLimitBucket {
                window_started_at: now,
                request_count: 0,
            });

        let elapsed = now.duration_since(bucket.window_started_at);
        if elapsed >= self.window {
            bucket.window_started_at = now;
            bucket.request_count = 0;
        }

        if bucket.request_count >= self.limit {
            let retry_after_seconds = self
                .window
                .saturating_sub(now.duration_since(bucket.window_started_at))
                .as_secs()
                .max(1);

            return RateLimitDecision::Limited {
                retry_after_seconds,
            };
        }

        bucket.request_count += 1;
        RateLimitDecision::Allowed
    }
}

struct RateLimitBucket {
    window_started_at: Instant,
    request_count: u32,
}

pub enum RateLimitDecision {
    Allowed,
    Limited { retry_after_seconds: u64 },
}

pub async fn enforce_send_rate_limit(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let client_key = client_key(request.headers());

    match state.rate_limiter.check(&client_key) {
        RateLimitDecision::Allowed => next.run(request).await,
        RateLimitDecision::Limited {
            retry_after_seconds,
        } => (
            StatusCode::TOO_MANY_REQUESTS,
            [(header::RETRY_AFTER, retry_after_seconds.to_string())],
            Json(ErrorResponse::rate_limited()),
        )
            .into_response(),
    }
}

fn client_key(headers: &HeaderMap) -> String {
    forwarded_for(headers)
        .or_else(|| header_value(headers, "x-real-ip"))
        .unwrap_or_else(|| "unknown-client".to_string())
}

fn forwarded_for(headers: &HeaderMap) -> Option<String> {
    header_value(headers, "x-forwarded-for").and_then(|value| {
        value
            .split(',')
            .map(str::trim)
            .find(|part| !part.is_empty())
            .map(ToOwned::to_owned)
    })
}

fn header_value(headers: &HeaderMap, name: &'static str) -> Option<String> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

#[cfg(test)]
mod tests {
    use std::{thread, time::Duration};

    use super::{RateLimitDecision, RateLimiter};

    #[test]
    fn allows_requests_under_limit() {
        let limiter = RateLimiter::new(2, Duration::from_secs(60));

        assert!(matches!(limiter.check("192.0.2.10"), RateLimitDecision::Allowed));
        assert!(matches!(limiter.check("192.0.2.10"), RateLimitDecision::Allowed));
    }

    #[test]
    fn blocks_requests_over_limit() {
        let limiter = RateLimiter::new(2, Duration::from_secs(60));

        assert!(matches!(limiter.check("192.0.2.10"), RateLimitDecision::Allowed));
        assert!(matches!(limiter.check("192.0.2.10"), RateLimitDecision::Allowed));
        assert!(matches!(
            limiter.check("192.0.2.10"),
            RateLimitDecision::Limited { .. }
        ));
    }

    #[test]
    fn resets_after_window() {
        let limiter = RateLimiter::new(1, Duration::from_millis(10));

        assert!(matches!(limiter.check("192.0.2.10"), RateLimitDecision::Allowed));
        assert!(matches!(
            limiter.check("192.0.2.10"),
            RateLimitDecision::Limited { .. }
        ));

        thread::sleep(Duration::from_millis(15));

        assert!(matches!(limiter.check("192.0.2.10"), RateLimitDecision::Allowed));
    }
}
