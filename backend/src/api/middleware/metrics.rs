#![allow(dead_code)]

use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use std::time::Instant;

use crate::domain::services::MetricsService;

/// Metrics middleware that tracks HTTP request metrics
pub async fn metrics_middleware(
    metrics: Arc<MetricsService>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let start_time = Instant::now();
    let method = req.method().to_string();
    let uri = req.uri().path().to_string();

    // Increment active requests
    metrics.http_requests_active.inc();

    // Process the request
    let response = next.run(req).await;

    // Record metrics
    let duration = start_time.elapsed().as_secs_f64();
    metrics.http_requests_duration.observe(duration);
    metrics.http_requests_total.inc();
    metrics.http_requests_active.dec();

    // Record errors
    if response.status().is_server_error() || response.status().is_client_error() {
        metrics.record_error();
    }

    // Log the request
    tracing::info!(
        method = %method,
        uri = %uri,
        status = %response.status(),
        duration = ?duration,
        "HTTP request completed"
    );

    Ok(response)
}
