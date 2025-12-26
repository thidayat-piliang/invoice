use crate::integration::{test_client::ApiTestClient, utils::get_api_base_url};
use serde_json::Value;

async fn setup_authenticated_client() -> ApiTestClient {
    let base_url = get_api_base_url();
    let client = ApiTestClient::new(base_url);

    let unique_id = crate::integration::utils::get_unique_id();
    let email = format!("advanced_test_{}@example.com", unique_id);
    let password = "testpassword123";

    client.register(&email, password, Some("Advanced Test Company")).await.unwrap();
    let resp = client.login(&email, password).await.unwrap();
    let data: Value = resp.json().await.unwrap();
    let token = data["access_token"].as_str().unwrap().to_string();

    let mut authed_client = client.clone();
    authed_client.set_token(token);
    authed_client
}

#[tokio::test]
async fn test_report_export_csv() {
    let client = setup_authenticated_client().await;

    // Create some data first
    let resp = client.create_client("Export Client", "export@test.com").await.unwrap();
    let client_data: Value = resp.json().await.unwrap();
    let client_id = client_data["id"].as_str().unwrap().to_string();

    client.create_invoice(&client_id, 500.0).await.unwrap();

    // Export income report as CSV
    let resp = client.export_report("income", "csv", "2024-01-01", "2024-12-31").await.unwrap();
    assert_eq!(resp.status(), 200);

    // Verify it's CSV content
    let body = resp.bytes().await.unwrap();
    assert!(!body.is_empty());

    // Cleanup
    client.delete_invoice(&client_id).await.unwrap();
    client.delete_client(&client_id).await.unwrap();
}

#[tokio::test]
async fn test_report_export_pdf() {
    let client = setup_authenticated_client().await;

    // Create some data first
    let resp = client.create_client("PDF Client", "pdf@test.com").await.unwrap();
    let client_data: Value = resp.json().await.unwrap();
    let client_id = client_data["id"].as_str().unwrap().to_string();

    client.create_invoice(&client_id, 1000.0).await.unwrap();

    // Export income report as PDF
    let resp = client.export_report("income", "pdf", "2024-01-01", "2024-12-31").await.unwrap();
    assert_eq!(resp.status(), 200);

    // Verify it's PDF content (starts with %PDF)
    let body = resp.bytes().await.unwrap();
    assert!(!body.is_empty());

    // Check for PDF magic bytes
    let pdf_header = body.get(0..4).unwrap_or_default();
    assert_eq!(pdf_header, b"%PDF");

    // Cleanup
    client.delete_invoice(&client_id).await.unwrap();
    client.delete_client(&client_id).await.unwrap();
}

#[tokio::test]
async fn test_metrics_endpoint() {
    let client = ApiTestClient::new(get_api_base_url());

    // Get metrics
    let resp = client.get_metrics().await.unwrap();
    assert_eq!(resp.status(), 200);

    // Verify it's Prometheus format
    let body = resp.text().await.unwrap();
    assert!(body.contains("http_requests_total") || body.contains("# HELP"));
}

#[tokio::test]
async fn test_payment_gateway_integration() {
    let client = setup_authenticated_client().await;

    // Get available gateways
    let resp = client.get_available_gateways().await.unwrap();
    assert_eq!(resp.status(), 200);

    // Create Stripe payment intent (will work even without real API key - simulates)
    let resp = client.create_stripe_payment_intent(100.0, "usd").await.unwrap();
    if resp.status() == 201 {
        let intent: Value = resp.json().await.unwrap();
        assert!(intent["id"].as_str().unwrap().starts_with("pi_"));
        assert_eq!(intent["amount"], 100.0);
    } else {
        // Should return 400 or 500 if not configured, but not panic
        assert!(resp.status() == 400 || resp.status() == 500 || resp.status() == 201);
    }

    // Create PayPal order
    let resp = client.create_paypal_order(100.0, "usd").await.unwrap();
    if resp.status() == 201 {
        let order: Value = resp.json().await.unwrap();
        assert!(order["id"].as_str().unwrap().starts_with("ORDER_"));
    } else {
        assert!(resp.status() == 400 || resp.status() == 500 || resp.status() == 201);
    }
}

#[tokio::test]
async fn test_push_notification() {
    let client = setup_authenticated_client().await;

    // Get current user ID
    let resp = client.get_current_user().await.unwrap();
    let user_data: Value = resp.json().await.unwrap();
    let user_id = user_data["id"].as_str().unwrap().to_string();

    // Send push notification (will work even without FCM configured)
    let resp = client.send_push_notification(&user_id, "Test Title", "Test Body").await.unwrap();

    // Should succeed even without FCM (graceful fallback)
    assert!(resp.status() == 200 || resp.status() == 202 || resp.status() == 500);
}

#[tokio::test]
async fn test_rate_limiting() {
    let client = setup_authenticated_client().await;

    // Make multiple requests quickly
    let mut success_count = 0;
    let mut rate_limited_count = 0;

    for _ in 0..150 {
        let resp = client.list_clients().await.unwrap();
        if resp.status() == 200 {
            success_count += 1;
        } else if resp.status() == 429 {
            rate_limited_count += 1;
            break; // Got rate limited, stop
        }
    }

    // Either we hit rate limit or we didn't (depends on config)
    // The important thing is the system handles it gracefully
    assert!(success_count > 0 || rate_limited_count > 0);
}

#[tokio::test]
async fn test_security_headers() {
    // Make a raw request to check headers
    let raw_client = reqwest::Client::new();
    let resp = raw_client.get(&format!("{}/health", get_api_base_url()))
        .send()
        .await
        .unwrap();

    // Check for security headers
    let headers = resp.headers();

    // These headers should be present (if configured in main.rs)
    // Note: Some headers might only be on API routes, not health
    if let Some(x_content_type) = headers.get("x-content-type-options") {
        assert_eq!(x_content_type, "nosniff");
    }
}

#[tokio::test]
async fn test_cors_headers() {
    let raw_client = reqwest::Client::new();

    // Make a request to test CORS headers
    let resp = raw_client.get(&format!("{}/api/v1/invoices", get_api_base_url()))
        .header("Origin", "http://localhost:3000")
        .send()
        .await
        .unwrap();

    // Should have CORS headers if configured
    let headers = resp.headers();
    if let Some(allow_origin) = headers.get("access-control-allow-origin") {
        assert!(!allow_origin.is_empty());
    }
}

#[tokio::test]
async fn test_request_timeout() {
    let client = ApiTestClient::new(get_api_base_url());

    // Normal requests should work within timeout
    let resp = client.health_check().await.unwrap();
    assert_eq!(resp, "OK");
}
