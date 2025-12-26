//! Integration tests for FlashBill API
//!
//! These tests use a real HTTP client to test the entire API stack including:
//! - HTTP request/response handling
//! - Authentication/Authorization
//! - Database operations
//! - Business logic
//!
//! To run these tests, you need:
//! 1. A running PostgreSQL database
//! 2. Set DATABASE_URL environment variable
//! 3. Run the API server in test mode
//!
//! Example:
//! ```bash
//! export DATABASE_URL="postgres://postgres:postgres@localhost:5432/flashbill_test"
//! cargo run --bin flashbill-api &
//! cargo test --test integration
//! ```

pub mod test_client;
pub mod utils;
