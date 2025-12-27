#![allow(dead_code)]

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use crate::domain::services::RedisService;

/// Application health status
#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Degraded(String),
    Unhealthy(String),
}

/// System metrics
#[derive(Debug, Clone)]
pub struct SystemMetrics {
    pub total_requests: u64,
    pub failed_requests: u64,
    pub avg_response_time_ms: f64,
    pub db_connections: u32,
    pub redis_connected: bool,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
}

impl Default for SystemMetrics {
    fn default() -> Self {
        Self {
            total_requests: 0,
            failed_requests: 0,
            avg_response_time_ms: 0.0,
            db_connections: 0,
            redis_connected: false,
            memory_usage_mb: 0.0,
            cpu_usage_percent: 0.0,
        }
    }
}

/// Request tracker for monitoring
#[derive(Debug, Clone)]
pub struct RequestTracker {
    pub request_id: Uuid,
    pub method: String,
    pub path: String,
    pub start_time_ms: u128,
    pub user_id: Option<Uuid>,
}

impl RequestTracker {
    pub fn new(method: String, path: String, user_id: Option<Uuid>) -> Self {
        Self {
            request_id: Uuid::new_v4(),
            method,
            path,
            start_time_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis(),
            user_id,
        }
    }

    pub fn duration_ms(&self) -> f64 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        (now as f64 - self.start_time_ms as f64).max(0.0)
    }
}

/// Monitoring service for application observability
pub struct MonitoringService {
    // Atomic counters for thread-safe increments
    total_requests: AtomicU64,
    failed_requests: AtomicU64,
    successful_requests: AtomicU64,

    // Response time tracking
    total_response_time_ms: AtomicU64,

    // Active requests tracking
    active_requests: Arc<RwLock<Vec<RequestTracker>>>,

    // Error tracking
    error_log: Arc<RwLock<Vec<(chrono::DateTime<chrono::Utc>, String, String)>>>,

    // Health status
    health_status: Arc<RwLock<HealthStatus>>,
}

impl MonitoringService {
    pub fn new() -> Self {
        Self {
            total_requests: AtomicU64::new(0),
            failed_requests: AtomicU64::new(0),
            successful_requests: AtomicU64::new(0),
            total_response_time_ms: AtomicU64::new(0),
            active_requests: Arc::new(RwLock::new(Vec::new())),
            error_log: Arc::new(RwLock::new(Vec::new())),
            health_status: Arc::new(RwLock::new(HealthStatus::Healthy)),
        }
    }

    /// Record a successful request
    pub fn record_request(&self, duration_ms: f64) {
        self.total_requests.fetch_add(1, Ordering::SeqCst);
        self.successful_requests.fetch_add(1, Ordering::SeqCst);
        self.total_response_time_ms.fetch_add(duration_ms as u64, Ordering::SeqCst);
    }

    /// Record a failed request
    pub fn record_failure(&self, duration_ms: f64, error: String) {
        self.total_requests.fetch_add(1, Ordering::SeqCst);
        self.failed_requests.fetch_add(1, Ordering::SeqCst);
        self.total_response_time_ms.fetch_add(duration_ms as u64, Ordering::SeqCst);

        // Log error
        let error_log = self.error_log.clone();
        tokio::spawn(async move {
            let mut log = error_log.write().await;
            log.push((chrono::Utc::now(), error, "request".to_string()));
            // Keep only last 1000 errors
            if log.len() > 1000 {
                log.remove(0);
            }
        });
    }

    /// Track active request
    pub async fn start_request(&self, tracker: RequestTracker) {
        let mut active = self.active_requests.write().await;
        active.push(tracker);
    }

    /// End active request tracking
    pub async fn end_request(&self, request_id: Uuid) {
        let mut active = self.active_requests.write().await;
        active.retain(|t| t.request_id != request_id);
    }

    /// Get current metrics
    pub fn get_metrics(&self) -> SystemMetrics {
        let total = self.total_requests.load(Ordering::SeqCst);
        let failed = self.failed_requests.load(Ordering::SeqCst);
        let total_time = self.total_response_time_ms.load(Ordering::SeqCst);

        let avg_response = if total > 0 {
            total_time as f64 / total as f64
        } else {
            0.0
        };

        SystemMetrics {
            total_requests: total,
            failed_requests: failed,
            avg_response_time_ms: avg_response,
            db_connections: 0, // Will be updated by health check
            redis_connected: false, // Will be updated by health check
            memory_usage_mb: 0.0, // Will be updated by health check
            cpu_usage_percent: 0.0, // Will be updated by health check
        }
    }

    /// Get active requests
    pub async fn get_active_requests(&self) -> Vec<RequestTracker> {
        let active = self.active_requests.read().await;
        active.iter().cloned().collect()
    }

    /// Get recent errors
    pub async fn get_recent_errors(&self, limit: usize) -> Vec<(chrono::DateTime<chrono::Utc>, String, String)> {
        let log = self.error_log.read().await;
        log.iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    /// Get error count in last N minutes
    pub async fn get_error_count(&self, minutes: i64) -> usize {
        let log = self.error_log.read().await;
        let cutoff = chrono::Utc::now() - chrono::Duration::minutes(minutes);
        log.iter()
            .filter(|(timestamp, _, _)| *timestamp > cutoff)
            .count()
    }

    /// Update health status
    pub async fn set_health_status(&self, status: HealthStatus) {
        let mut health = self.health_status.write().await;
        *health = status;
    }

    /// Get current health status
    pub async fn get_health_status(&self) -> HealthStatus {
        let health = self.health_status.read().await;
        health.clone()
    }

    /// Perform health check
    pub async fn perform_health_check(
        &self,
        db_pool: Option<&sqlx::PgPool>,
        redis: Option<&Arc<RedisService>>,
    ) -> HealthStatus {
        let mut issues = Vec::new();

        // Check database
        if let Some(pool) = db_pool {
            match sqlx::query("SELECT 1").fetch_one(pool).await {
                Ok(_) => {}
                Err(e) => issues.push(format!("Database: {}", e)),
            }
        }

        // Check Redis
        if let Some(redis_service) = redis {
            match redis_service.as_ref().exists("health:check").await {
                Ok(_) => {}
                Err(e) => issues.push(format!("Redis: {}", e)),
            }
        }

        // Check error rate (if more than 10 errors in last 5 minutes, degraded)
        let error_count = self.get_error_count(5).await;
        if error_count > 10 {
            issues.push(format!("High error rate: {} errors in last 5 minutes", error_count));
        }

        if issues.is_empty() {
            HealthStatus::Healthy
        } else if issues.len() <= 2 {
            HealthStatus::Degraded(issues.join(", "))
        } else {
            HealthStatus::Unhealthy(issues.join(", "))
        }
    }

    /// Get performance summary
    pub async fn get_performance_summary(&self) -> PerformanceSummary {
        let metrics = self.get_metrics();
        let active_count = self.active_requests.read().await.len();
        let error_count = self.get_error_count(1).await;

        PerformanceSummary {
            total_requests: metrics.total_requests,
            failed_requests: metrics.failed_requests,
            success_rate: if metrics.total_requests > 0 {
                ((metrics.total_requests - metrics.failed_requests) as f64 / metrics.total_requests as f64) * 100.0
            } else {
                100.0
            },
            avg_response_time_ms: metrics.avg_response_time_ms,
            active_requests: active_count,
            errors_last_minute: error_count,
        }
    }

    /// Log custom event
    pub async fn log_event(&self, event_type: &str, message: &str) {
        let mut log = self.error_log.write().await;
        log.push((chrono::Utc::now(), format!("[{}] {}", event_type, message), event_type.to_string()));
        if log.len() > 1000 {
            log.remove(0);
        }
    }

    /// Get throughput (requests per second)
    pub fn get_throughput(&self) -> f64 {
        // This is a simplified calculation
        // In production, you'd track timestamps and calculate over a time window
        let metrics = self.get_metrics();
        if metrics.avg_response_time_ms > 0.0 {
            1000.0 / metrics.avg_response_time_ms
        } else {
            0.0
        }
    }
}

#[derive(Debug)]
pub struct PerformanceSummary {
    pub total_requests: u64,
    pub failed_requests: u64,
    pub success_rate: f64,
    pub avg_response_time_ms: f64,
    pub active_requests: usize,
    pub errors_last_minute: usize,
}

/// Request duration guard
pub struct RequestDurationGuard {
    tracker: RequestTracker,
    monitoring: Arc<MonitoringService>,
    is_success: bool,
}

impl RequestDurationGuard {
    pub fn new(monitoring: Arc<MonitoringService>, method: String, path: String, user_id: Option<Uuid>) -> Self {
        let tracker = RequestTracker::new(method, path, user_id);
        let monitoring_clone = monitoring.clone();
        let tracker_clone = tracker.clone();

        // Start tracking
        tokio::spawn(async move {
            monitoring_clone.start_request(tracker_clone).await;
        });

        Self {
            tracker,
            monitoring,
            is_success: true,
        }
    }

    pub fn mark_failed(&mut self) {
        self.is_success = false;
    }

    pub fn finish(self) {
        let duration = self.tracker.duration_ms();
        let monitoring = self.monitoring.clone();
        let is_success = self.is_success;
        let request_id = self.tracker.request_id;

        tokio::spawn(async move {
            if is_success {
                monitoring.record_request(duration);
            } else {
                monitoring.record_failure(duration, "Request failed".to_string());
            }
            monitoring.end_request(request_id).await;
        });
    }
}

impl Drop for RequestDurationGuard {
    fn drop(&mut self) {
        // Always finish tracking on drop
        let duration = self.tracker.duration_ms();
        let monitoring = self.monitoring.clone();
        let is_success = self.is_success;
        let request_id = self.tracker.request_id;

        tokio::spawn(async move {
            if is_success {
                monitoring.record_request(duration);
            } else {
                monitoring.record_failure(duration, "Request failed".to_string());
            }
            monitoring.end_request(request_id).await;
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_monitoring_service() {
        let monitor = Arc::new(MonitoringService::new());

        // Record some requests
        monitor.record_request(10.0);
        monitor.record_request(20.0);
        monitor.record_failure(15.0, "Test error".to_string());

        let metrics = monitor.get_metrics();
        assert_eq!(metrics.total_requests, 3);
        assert_eq!(metrics.failed_requests, 1);
        assert_eq!(metrics.avg_response_time_ms, 15.0);

        // Test health status
        monitor.set_health_status(HealthStatus::Degraded("Test".to_string())).await;
        let status = monitor.get_health_status().await;
        assert!(matches!(status, HealthStatus::Degraded(_)));
    }

    #[tokio::test]
    async fn test_request_guard() {
        let monitor = Arc::new(MonitoringService::new());

        {
            let _guard = RequestDurationGuard::new(
                monitor.clone(),
                "GET".to_string(),
                "/test".to_string(),
                None,
            );
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        } // Guard is dropped here

        // Give it a moment to process
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        let metrics = monitor.get_metrics();
        assert_eq!(metrics.total_requests, 1);
    }
}
