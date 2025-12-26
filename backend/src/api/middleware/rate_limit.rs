use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use governor::{
    clock::DefaultClock,
    middleware::RateLimitingMiddleware,
    state::{InMemoryState, keyed::DefaultKeyedStateStore},
    Quota, RateLimiter,
};
use nonzero_ext::nonzero;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
pub struct RateLimitMiddleware {
    limiter: Arc<RateLimiter<String, DefaultKeyedStateStore<String>, DefaultClock>>,
}

impl RateLimitMiddleware {
    pub fn new() -> Self {
        // 100 requests per minute per IP
        let quota = Quota::per_minute(nonzero!(100u32));
        let limiter = Arc::new(RateLimiter::<
            String,
            DefaultKeyedStateStore<String>,
            DefaultClock,
        >::keyed(quota));

        Self { limiter }
    }

    pub async fn check_rate_limit(&self, key: &str) -> bool {
        self.limiter.check_key(&key.to_string()).is_ok()
    }
}

pub async fn rate_limit_middleware(
    req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract IP address from request
    let ip = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.to_string())
        .or_else(|| {
            req.extensions()
                .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
                .map(|ci| ci.0.ip().to_string())
        })
        .unwrap_or_else(|| "unknown".to_string());

    // Check rate limit
    let rate_limiter = RateLimitMiddleware::new();
    if !rate_limiter.check_rate_limit(&ip).await {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    Ok(next.run(req).await)
}
