use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use std::sync::Arc;

use crate::domain::services::MetricsService;

#[derive(Clone)]
struct MetricsState {
    metrics: Arc<MetricsService>,
}

pub fn create_router(metrics: Arc<MetricsService>) -> Router {
    let state = MetricsState { metrics };

    Router::new()
        .route("/metrics", get(get_metrics))
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
