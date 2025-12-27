use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response, Json},
    routing::get,
    Router,
};
use std::sync::Arc;
use serde_json::json;

use crate::domain::services::{MetricsService, MonitoringService, RedisService};

#[derive(Clone)]
struct MetricsState {
    metrics: Arc<MetricsService>,
    monitoring: Arc<MonitoringService>,
    redis: Option<Arc<RedisService>>,
    db_pool: Option<sqlx::PgPool>,
}

pub fn create_router(
    metrics: Arc<MetricsService>,
    monitoring: Arc<MonitoringService>,
    redis: Option<Arc<RedisService>>,
    db_pool: Option<sqlx::PgPool>,
) -> Router {
    let state = MetricsState {
        metrics,
        monitoring,
        redis,
        db_pool,
    };

    Router::new()
        .route("/metrics", get(get_metrics))
        .route("/health", get(health_check))
        .route("/ready", get(readiness_check))
        .route("/monitoring/summary", get(get_monitoring_summary))
        .route("/monitoring/active-requests", get(get_active_requests))
        .route("/monitoring/errors", get(get_recent_errors))
        .with_state(state)
}

async fn get_metrics(State(state): State<MetricsState>) -> Result<Response, StatusCode> {
    match state.metrics.get_metrics() {
        Ok(metrics) => Ok((
            [(axum::http::header::CONTENT_TYPE, "text/plain; version=0.0.4")],
            metrics,
        )
            .into_response()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Health check endpoint - returns 200 if service is running
async fn health_check(State(_state): State<MetricsState>) -> StatusCode {
    // Basic health check - just check if we can respond
    StatusCode::OK
}

/// Readiness check - performs deeper health checks
async fn readiness_check(State(state): State<MetricsState>) -> (StatusCode, Json<serde_json::Value>) {
    let health_status = state.monitoring.perform_health_check(
        state.db_pool.as_ref(),
        state.redis.as_ref(),
    ).await;

    match health_status {
        crate::domain::services::HealthStatus::Healthy => {
            (StatusCode::OK, Json(json!({
                "status": "healthy",
                "message": "All systems operational"
            })))
        }
        crate::domain::services::HealthStatus::Degraded(msg) => {
            (StatusCode::OK, Json(json!({
                "status": "degraded",
                "message": msg,
                "warning": "Service is operational but with degraded performance"
            })))
        }
        crate::domain::services::HealthStatus::Unhealthy(msg) => {
            (StatusCode::SERVICE_UNAVAILABLE, Json(json!({
                "status": "unhealthy",
                "message": msg,
                "error": "Service is not healthy"
            })))
        }
    }
}

/// Get monitoring summary
async fn get_monitoring_summary(State(state): State<MetricsState>) -> (StatusCode, Json<serde_json::Value>) {
    let summary = state.monitoring.get_performance_summary().await;
    let metrics = state.monitoring.get_metrics();

    (StatusCode::OK, Json(json!({
        "total_requests": summary.total_requests,
        "failed_requests": summary.failed_requests,
        "success_rate": summary.success_rate,
        "avg_response_time_ms": summary.avg_response_time_ms,
        "active_requests": summary.active_requests,
        "errors_last_minute": summary.errors_last_minute,
        "throughput_per_second": state.monitoring.get_throughput(),
        "metrics": {
            "db_connections": metrics.db_connections,
            "redis_connected": metrics.redis_connected,
        }
    })))
}

/// Get active requests
async fn get_active_requests(State(state): State<MetricsState>) -> (StatusCode, Json<serde_json::Value>) {
    let active = state.monitoring.get_active_requests().await;

    let requests: Vec<_> = active.iter().map(|req| {
        json!({
            "request_id": req.request_id.to_string(),
            "method": req.method,
            "path": req.path,
            "duration_ms": req.duration_ms(),
            "user_id": req.user_id.map(|u| u.to_string()),
        })
    }).collect();

    (StatusCode::OK, Json(json!({
        "active_count": requests.len(),
        "requests": requests
    })))
}

/// Get recent errors
async fn get_recent_errors(State(state): State<MetricsState>) -> (StatusCode, Json<serde_json::Value>) {
    let errors = state.monitoring.get_recent_errors(50).await;

    let errors_json: Vec<_> = errors.iter().map(|(timestamp, error, error_type)| {
        json!({
            "timestamp": timestamp.to_string(),
            "error": error,
            "type": error_type,
        })
    }).collect();

    (StatusCode::OK, Json(json!({
        "count": errors_json.len(),
        "errors": errors_json
    })))
}
