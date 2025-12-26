use prometheus::{
    register_int_counter, register_int_gauge, register_histogram,
    IntCounter, IntGauge, Histogram, Encoder, TextEncoder, Registry
};
use std::sync::Arc;

/// Service for collecting and exposing Prometheus metrics
#[derive(Clone)]
pub struct MetricsService {
    registry: Arc<Registry>,

    // Request metrics
    pub http_requests_total: IntCounter,
    pub http_requests_duration: Histogram,
    pub http_requests_active: IntGauge,

    // Business metrics
    pub invoices_created_total: IntCounter,
    pub invoices_paid_total: IntCounter,
    pub invoices_overdue_total: IntGauge,
    pub expenses_total: IntCounter,
    pub payments_processed_total: IntCounter,

    // Cache metrics
    pub cache_hits_total: IntCounter,
    pub cache_misses_total: IntCounter,

    // Error metrics
    pub errors_total: IntCounter,
}

impl MetricsService {
    pub fn new() -> Result<Self, prometheus::Error> {
        let registry = Arc::new(Registry::new());

        // HTTP request metrics
        let http_requests_total = register_int_counter!(
            "http_requests_total",
            "Total number of HTTP requests"
        )?;

        let http_requests_duration = register_histogram!(
            "http_request_duration_seconds",
            "HTTP request duration in seconds",
            vec![0.001, 0.01, 0.1, 0.5, 1.0, 2.0, 5.0]
        )?;

        let http_requests_active = register_int_gauge!(
            "http_requests_active",
            "Number of active HTTP requests"
        )?;

        // Business metrics
        let invoices_created_total = register_int_counter!(
            "invoices_created_total",
            "Total number of invoices created"
        )?;

        let invoices_paid_total = register_int_counter!(
            "invoices_paid_total",
            "Total number of invoices paid"
        )?;

        let invoices_overdue_total = register_int_gauge!(
            "invoices_overdue_total",
            "Number of currently overdue invoices"
        )?;

        let expenses_total = register_int_counter!(
            "expenses_total",
            "Total number of expenses recorded"
        )?;

        let payments_processed_total = register_int_counter!(
            "payments_processed_total",
            "Total number of payments processed"
        )?;

        // Cache metrics
        let cache_hits_total = register_int_counter!(
            "cache_hits_total",
            "Total number of cache hits"
        )?;

        let cache_misses_total = register_int_counter!(
            "cache_misses_total",
            "Total number of cache misses"
        )?;

        // Error metrics
        let errors_total = register_int_counter!(
            "errors_total",
            "Total number of errors"
        )?;

        Ok(Self {
            registry,
            http_requests_total,
            http_requests_duration,
            http_requests_active,
            invoices_created_total,
            invoices_paid_total,
            invoices_overdue_total,
            expenses_total,
            payments_processed_total,
            cache_hits_total,
            cache_misses_total,
            errors_total,
        })
    }

    /// Get metrics in Prometheus text format
    pub fn get_metrics(&self) -> Result<String, prometheus::Error> {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer)?;
        String::from_utf8(buffer).map_err(|e| prometheus::Error::Msg(e.to_string()))
    }

    /// Record a cache hit
    pub fn record_cache_hit(&self) {
        self.cache_hits_total.inc();
    }

    /// Record a cache miss
    pub fn record_cache_miss(&self) {
        self.cache_misses_total.inc();
    }

    /// Record an invoice creation
    pub fn record_invoice_created(&self) {
        self.invoices_created_total.inc();
    }

    /// Record an invoice payment
    pub fn record_invoice_paid(&self) {
        self.invoices_paid_total.inc();
    }

    /// Update overdue invoice count
    pub fn set_overdue_invoices(&self, count: i64) {
        self.invoices_overdue_total.set(count);
    }

    /// Record an expense
    pub fn record_expense(&self) {
        self.expenses_total.inc();
    }

    /// Record a payment
    pub fn record_payment(&self) {
        self.payments_processed_total.inc();
    }

    /// Record an error
    pub fn record_error(&self) {
        self.errors_total.inc();
    }
}

impl Default for MetricsService {
    fn default() -> Self {
        Self::new().expect("Failed to initialize metrics service")
    }
}
