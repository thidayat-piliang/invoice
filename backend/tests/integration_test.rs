//! FlashBill API Integration Tests
//!
//! These tests require:
//! 1. A running PostgreSQL database for testing
//! 2. The FlashBill API server running on localhost:3000 (or set API_URL)
//!
//! Setup instructions:
//! ```bash
//! # Create test database
//! createdb flashbill_test
//!
//! # Set environment variables
//! export DATABASE_URL="postgres://postgres:postgres@localhost:5432/flashbill_test"
//! export API_URL="http://localhost:3000"
//!
//! # Run migrations
//! cargo run --bin flashbill-api
//!
//! # In another terminal, run tests
//! cargo test --test integration_test
//! ```

mod integration {
    pub mod test_client;
    pub mod utils;
    pub mod auth_test;
    pub mod clients_test;
    pub mod invoices_test;
    pub mod payments_test;
    pub mod expenses_test;
    pub mod reports_test;
    pub mod settings_test;
}

#[cfg(test)]
mod tests {
    use crate::integration::utils::{get_api_base_url, wait_for_api};

    #[tokio::test]
    async fn test_api_is_running() {
        let base_url = get_api_base_url();
        let is_ready = wait_for_api(&base_url, 10).await;
        assert!(is_ready, "API server should be running at {}", base_url);
    }
}
