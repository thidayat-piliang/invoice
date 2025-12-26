use std::env;
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::time::{SystemTime, UNIX_EPOCH};

/// Get database URL from environment or use default for testing
pub fn get_test_db_url() -> String {
    env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/flashbill_test".to_string())
}

/// Get API base URL from environment or use default
pub fn get_api_base_url() -> String {
    env::var("API_URL")
        .unwrap_or_else(|_| "http://localhost:3000".to_string())
}

/// Generate a unique ID for test data to avoid collisions
pub fn get_unique_id() -> String {
    let start = SystemTime::now();
    let since_epoch = start.duration_since(UNIX_EPOCH).expect("Time went backwards");
    // Use nanoseconds for uniqueness
    format!("{}_{}", since_epoch.as_secs(), since_epoch.subsec_nanos())
}

/// Create a test database pool
pub async fn create_test_pool() -> PgPool {
    let db_url = get_test_db_url();
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("Failed to create test database pool")
}

/// Clean up test data
pub async fn cleanup_test_data(pool: &PgPool) {
    // Delete all test data
    let _ = sqlx::query(
        r#"
        DELETE FROM expenses;
        DELETE FROM payments;
        DELETE FROM invoices;
        DELETE FROM clients;
        DELETE FROM users;
        "#
    )
    .execute(pool)
    .await;
}

/// Wait for API to be ready
pub async fn wait_for_api(base_url: &str, max_retries: u32) -> bool {
    use std::time::Duration;
    use tokio::time::sleep;

    for _ in 0..max_retries {
        let client = reqwest::Client::new();
        match client.get(&format!("{}/health", base_url)).send().await {
            Ok(resp) if resp.status().is_success() => return true,
            _ => sleep(Duration::from_secs(1)).await,
        }
    }
    false
}
