#![allow(dead_code)]

use std::time::Duration;
use tokio::time::sleep;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RetryError {
    #[error("Operation failed after {0} attempts")]
    MaxRetriesExceeded(u32),
    #[error("Operation cancelled")]
    Cancelled,
}

/// Configuration for retry behavior
#[derive(Clone, Debug)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub exponential_base: f64,
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 100,
            max_delay_ms: 5000,
            exponential_base: 2.0,
            jitter: true,
        }
    }
}

impl RetryConfig {
    /// Conservative config for database operations
    pub fn database() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 50,
            max_delay_ms: 2000,
            exponential_base: 2.0,
            jitter: true,
        }
    }

    /// Aggressive config for network operations
    pub fn network() -> Self {
        Self {
            max_retries: 5,
            initial_delay_ms: 200,
            max_delay_ms: 10000,
            exponential_base: 2.0,
            jitter: true,
        }
    }

    /// Gentle config for non-critical operations
    pub fn gentle() -> Self {
        Self {
            max_retries: 2,
            initial_delay_ms: 100,
            max_delay_ms: 1000,
            exponential_base: 1.5,
            jitter: false,
        }
    }

    /// Calculate delay for retry attempt
    fn calculate_delay(&self, attempt: u32) -> Duration {
        let base = self.initial_delay_ms as f64;
        let exponent = self.exponential_base.powf(attempt as f64 - 1.0);
        let mut delay_ms = (base * exponent).min(self.max_delay_ms as f64) as u64;

        // Add jitter (random variation) to prevent thundering herd
        if self.jitter {
            use rand::Rng;
            let mut rng = rand::rng();
            let jitter_factor: f64 = rng.random_range(0.8..1.2);
            delay_ms = (delay_ms as f64 * jitter_factor) as u64;
        }

        Duration::from_millis(delay_ms)
    }
}

/// Retry utility with exponential backoff and jitter
pub struct RetryService {
    config: RetryConfig,
}

impl RetryService {
    pub fn new(config: RetryConfig) -> Self {
        Self { config }
    }

    pub fn with_default() -> Self {
        Self::new(RetryConfig::default())
    }

    /// Execute an async operation with retry logic
    pub async fn execute<T, E, F, Fut>(&self, operation: F) -> Result<T, RetryError>
    where
        F: Fn(u32) -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Display,
    {
        for attempt in 1..=self.config.max_retries {
            match operation(attempt).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    // If this was the last attempt, return error
                    if attempt == self.config.max_retries {
                        break;
                    }

                    // Check if error is retryable
                    if !self.is_retryable_error(&e.to_string()) {
                        return Err(RetryError::MaxRetriesExceeded(attempt));
                    }

                    // Calculate delay and wait
                    let delay = self.config.calculate_delay(attempt);
                    tracing::warn!(
                        attempt = attempt,
                        max_retries = self.config.max_retries,
                        delay_ms = delay.as_millis(),
                        error = %e,
                        "Operation failed, retrying..."
                    );

                    sleep(delay).await;
                }
            }
        }

        Err(RetryError::MaxRetriesExceeded(self.config.max_retries))
    }

    /// Execute with custom retry condition
    pub async fn execute_with_condition<T, E, F, Fut, Cond>(
        &self,
        operation: F,
        should_retry: Cond,
    ) -> Result<T, RetryError>
    where
        F: Fn(u32) -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Display,
        Cond: Fn(&E) -> bool,
    {
        for attempt in 1..=self.config.max_retries {
            match operation(attempt).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    // Check custom condition
                    if !should_retry(&e) {
                        return Err(RetryError::MaxRetriesExceeded(attempt));
                    }

                    if attempt == self.config.max_retries {
                        break;
                    }

                    let delay = self.config.calculate_delay(attempt);
                    tracing::warn!(
                        attempt = attempt,
                        delay_ms = delay.as_millis(),
                        error = %e,
                        "Operation failed, retrying..."
                    );

                    sleep(delay).await;
                }
            }
        }

        Err(RetryError::MaxRetriesExceeded(self.config.max_retries))
    }

    /// Check if an error is worth retrying
    fn is_retryable_error(&self, error: &str) -> bool {
        error.to_lowercase().contains("connection")
            || error.to_lowercase().contains("timeout")
            || error.to_lowercase().contains("network")
            || error.to_lowercase().contains("dns")
            || error.to_lowercase().contains("refused")
            || error.to_lowercase().contains("reset")
            || error.to_lowercase().contains("unavailable")
            || error.to_lowercase().contains("busy")
            || error.to_lowercase().contains("deadlock")
            || error.to_lowercase().contains("lock")
            || error.to_lowercase().contains("rate limit")
            || error.contains("429")
            || error.contains("500")
            || error.contains("502")
            || error.contains("503")
            || error.contains("504")
    }

    /// Execute with timeout
    pub async fn execute_with_timeout<T, E, F, Fut>(
        &self,
        operation: F,
        timeout_duration: Duration,
    ) -> Result<T, RetryError>
    where
        F: Fn(u32) -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Display,
    {
        use tokio::time::timeout;

        for attempt in 1..=self.config.max_retries {
            let result = timeout(timeout_duration, operation(attempt)).await;

            match result {
                Ok(Ok(value)) => return Ok(value),
                Ok(Err(e)) => {
                    if attempt == self.config.max_retries {
                        break;
                    }

                    if !self.is_retryable_error(&e.to_string()) {
                        return Err(RetryError::MaxRetriesExceeded(attempt));
                    }

                    let delay = self.config.calculate_delay(attempt);
                    tracing::warn!(
                        attempt = attempt,
                        delay_ms = delay.as_millis(),
                        error = %e,
                        "Operation failed with timeout, retrying..."
                    );

                    sleep(delay).await;
                }
                Err(_timeout) => {
                    if attempt == self.config.max_retries {
                        break;
                    }

                    let delay = self.config.calculate_delay(attempt);
                    tracing::warn!(
                        attempt = attempt,
                        delay_ms = delay.as_millis(),
                        "Operation timed out, retrying..."
                    );

                    sleep(delay).await;
                }
            }
        }

        Err(RetryError::MaxRetriesExceeded(self.config.max_retries))
    }
}

/// Convenience macro for retry operations
#[macro_export]
macro_rules! retry {
    ($operation:expr) => {
        $crate::domain::services::retry_service::RetryService::with_default()
            .execute(|attempt| async move { $operation })
            .await
    };
    ($operation:expr, $config:expr) => {
        $crate::domain::services::retry_service::RetryService::new($config)
            .execute(|attempt| async move { $operation })
            .await
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_retry_success_first_attempt() {
        let service = RetryService::with_default();
        let result = service.execute(|_| async { Ok::<_, String>("success") }).await;
        assert_eq!(result.unwrap(), "success");
    }

    #[tokio::test]
    async fn test_retry_eventual_success() {
        let service = RetryService::new(RetryConfig {
            max_retries: 3,
            initial_delay_ms: 1,
            max_delay_ms: 10,
            exponential_base: 1.0,
            jitter: false,
        });

        let mut attempts = 0;
        let result = service.execute(|_| {
            attempts += 1;
            async move {
                if attempts < 3 {
                    Err("temporary error")
                } else {
                    Ok("success")
                }
            }
        }).await;

        assert_eq!(result.unwrap(), "success");
        assert_eq!(attempts, 3);
    }

    #[tokio::test]
    async fn test_retry_max_exceeded() {
        let service = RetryService::new(RetryConfig {
            max_retries: 2,
            initial_delay_ms: 1,
            max_delay_ms: 10,
            exponential_base: 1.0,
            jitter: false,
        });

        let result = service.execute(|_| async { Err::<String, _>("always fails") }).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_retry_non_retryable_error() {
        let service = RetryService::with_default();
        let result = service.execute(|_| async { Err::<String, _>("validation error") }).await;
        assert!(result.is_err());
    }
}
