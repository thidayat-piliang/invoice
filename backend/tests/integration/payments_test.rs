use crate::integration::{test_client::ApiTestClient, utils::get_api_base_url};
use serde_json::Value;

async fn setup_authenticated_client() -> ApiTestClient {
    let base_url = get_api_base_url();
    let client = ApiTestClient::new(base_url);

    let unique_id = chrono::Utc::now().timestamp();
    let email = format!("payment_test_{}@example.com", unique_id);
    let password = "testpassword123";

    client.register(&email, password, Some("Test Company")).await.unwrap();
    let resp = client.login(&email, password).await.unwrap();
    let data: Value = resp.json().await.unwrap();
    let token = data["access_token"].as_str().unwrap().to_string();

    let mut authed_client = client.clone();
    authed_client.set_token(token);
    authed_client
}

#[tokio::test]
async fn test_payment_crud() {
    let client = setup_authenticated_client().await;

    // Setup: Create client and invoice
    let resp = client.create_client("Payment Client", "payment@test.com").await.unwrap();
    let client_data: Value = resp.json().await.unwrap();
    let client_id = client_data["id"].as_str().unwrap().to_string();

    let resp = client.create_invoice(&client_id, 300.0).await.unwrap();
    let invoice: Value = resp.json().await.unwrap();
    let invoice_id = invoice["id"].as_str().unwrap().to_string();

    // 1. Create payment
    let resp = client.create_payment(&invoice_id, 300.0).await.unwrap();
    assert_eq!(resp.status(), 201);
    let payment: Value = resp.json().await.unwrap();
    let payment_id = payment["id"].as_str().unwrap().to_string();
    assert_eq!(payment["amount"], 300.0);
    assert_eq!(payment["status"], "completed");

    // 2. Get payment
    let resp = client.get_payment(&payment_id).await.unwrap();
    assert_eq!(resp.status(), 200);
    let fetched: Value = resp.json().await.unwrap();
    assert_eq!(fetched["id"], payment_id);

    // 3. List payments
    let resp = client.list_payments().await.unwrap();
    assert_eq!(resp.status(), 200);
    let list: Value = resp.json().await.unwrap();
    assert!(list.is_array());
    assert!(list.as_array().unwrap().len() > 0);

    // 4. Get payment stats
    let resp = client.get_payment_stats().await.unwrap();
    assert_eq!(resp.status(), 200);
    let stats: Value = resp.json().await.unwrap();
    assert!(stats["total_payments"].is_number());

    // 5. Get payment methods
    let resp = client.get_payment_methods().await.unwrap();
    assert_eq!(resp.status(), 200);
    let methods: Value = resp.json().await.unwrap();
    assert!(methods.is_array());
    assert!(methods.as_array().unwrap().len() > 0);

    // Cleanup
    client.delete_invoice(&invoice_id).await.unwrap();
    client.delete_client(&client_id).await.unwrap();
}

#[tokio::test]
async fn test_refund_payment() {
    let client = setup_authenticated_client().await;

    // Setup
    let resp = client.create_client("Refund Client", "refund@test.com").await.unwrap();
    let client_data: Value = resp.json().await.unwrap();
    let client_id = client_data["id"].as_str().unwrap().to_string();

    let resp = client.create_invoice(&client_id, 500.0).await.unwrap();
    let invoice: Value = resp.json().await.unwrap();
    let invoice_id = invoice["id"].as_str().unwrap().to_string();

    // Create payment
    let resp = client.create_payment(&invoice_id, 500.0).await.unwrap();
    let payment: Value = resp.json().await.unwrap();
    let payment_id = payment["id"].as_str().unwrap().to_string();

    // Refund payment
    let resp = client.refund_payment(&payment_id, "Customer requested refund").await.unwrap();
    assert_eq!(resp.status(), 200);
    let refund: Value = resp.json().await.unwrap();
    assert_eq!(refund["status"], "refunded");

    // Cleanup
    client.delete_invoice(&invoice_id).await.unwrap();
    client.delete_client(&client_id).await.unwrap();
}

#[tokio::test]
async fn test_partial_payment() {
    let client = setup_authenticated_client().await;

    // Setup
    let resp = client.create_client("Partial Client", "partial@test.com").await.unwrap();
    let client_data: Value = resp.json().await.unwrap();
    let client_id = client_data["id"].as_str().unwrap().to_string();

    let resp = client.create_invoice(&client_id, 1000.0).await.unwrap();
    let invoice: Value = resp.json().await.unwrap();
    let invoice_id = invoice["id"].as_str().unwrap().to_string();

    // Record partial payment
    let resp = client.record_payment(&invoice_id, 500.0).await.unwrap();
    assert_eq!(resp.status(), 201);

    // Get invoice to check amount paid
    let resp = client.get_invoice(&invoice_id).await.unwrap();
    let updated_invoice: Value = resp.json().await.unwrap();
    assert_eq!(updated_invoice["amount_paid"], 500.0);
    assert_eq!(updated_invoice["status"], "partially_paid");

    // Cleanup
    client.delete_invoice(&invoice_id).await.unwrap();
    client.delete_client(&client_id).await.unwrap();
}

#[tokio::test]
async fn test_payment_with_different_methods() {
    let client = setup_authenticated_client().await;

    // Setup
    let resp = client.create_client("Method Client", "method@test.com").await.unwrap();
    let client_data: Value = resp.json().await.unwrap();
    let client_id = client_data["id"].as_str().unwrap().to_string();

    let resp = client.create_invoice(&client_id, 200.0).await.unwrap();
    let invoice: Value = resp.json().await.unwrap();
    let invoice_id = invoice["id"].as_str().unwrap().to_string();

    // Test different payment methods
    let methods = ["stripe", "paypal", "check", "cash"];
    for method in methods {
        let mut request = client.clone();
        let resp = request.client.post(&format!("{}/api/v1/payments", get_api_base_url()))
            .header("Authorization", format!("Bearer {}", request.auth_token.unwrap()))
            .json(&serde_json::json!({
                "invoice_id": invoice_id,
                "amount": 50.0,
                "payment_method": method,
            }))
            .send()
            .await.unwrap();

        assert_eq!(resp.status(), 201, "Payment with {} should succeed", method);
    }

    // Cleanup
    client.delete_invoice(&invoice_id).await.unwrap();
    client.delete_client(&client_id).await.unwrap();
}
