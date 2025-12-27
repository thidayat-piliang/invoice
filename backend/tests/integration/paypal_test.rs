use crate::integration::{test_client::ApiTestClient, utils::get_api_base_url};
use serde_json::{json, Value};
use uuid::Uuid;

async fn setup_authenticated_client() -> ApiTestClient {
    let base_url = get_api_base_url();
    let client = ApiTestClient::new(base_url);

    let unique_id = crate::integration::utils::get_unique_id();
    let email = format!("paypal_test_{}@example.com", unique_id);
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
async fn test_paypal_create_order() {
    let client = setup_authenticated_client().await;

    // Create a PayPal order
    let resp = client.get_http_client().post(&format!("{}/api/v1/paypal/create-order", get_api_base_url()))
        .header("Authorization", format!("Bearer {}", client.get_auth_token().unwrap()))
        .json(&json!({
            "amount": 150.00,
            "currency": "USD",
            "description": "Test invoice payment",
            "invoice_id": Uuid::new_v4().to_string()
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 201);
    let order: serde_json::Value = resp.json().await.unwrap();

    // Verify response structure
    assert!(order["order_id"].as_str().is_some());
    assert_eq!(order["amount"], 150.00);
    assert_eq!(order["currency"], "USD");
    assert_eq!(order["payment_method"], "paypal");
    assert!(order["status"].as_str().is_some());
    assert!(order["checkout_url"].as_str().is_some());
    assert!(order["message"].as_str().is_some());
}

#[tokio::test]
async fn test_paypal_create_order_with_invalid_amount() {
    let client = setup_authenticated_client().await;

    // Try to create order with zero amount
    let resp = client.get_http_client().post(&format!("{}/api/v1/paypal/create-order", get_api_base_url()))
        .header("Authorization", format!("Bearer {}", client.get_auth_token().unwrap()))
        .json(&json!({
            "amount": 0.0,
            "currency": "USD"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 400);
}

#[tokio::test]
async fn test_paypal_refund() {
    let client = setup_authenticated_client().await;

    // First create an order
    let create_resp = client.get_http_client().post(&format!("{}/api/v1/paypal/create-order", get_api_base_url()))
        .header("Authorization", format!("Bearer {}", client.get_auth_token().unwrap()))
        .json(&json!({
            "amount": 100.00,
            "currency": "USD"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(create_resp.status(), 201);
    let order: serde_json::Value = create_resp.json().await.unwrap();
    let order_id = order["order_id"].as_str().unwrap();

    // Now refund it
    let refund_resp = client.get_http_client().post(&format!("{}/api/v1/paypal/refund", get_api_base_url()))
        .header("Authorization", format!("Bearer {}", client.get_auth_token().unwrap()))
        .json(&json!({
            "payment_id": order_id,
            "amount": 100.00,
            "reason": "Customer requested refund"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(refund_resp.status(), 200);
    let refund: serde_json::Value = refund_resp.json().await.unwrap();

    assert!(refund["refund_id"].as_str().is_some());
    assert_eq!(refund["amount"], 100.00);
    assert!(refund["status"].as_str().is_some());
    assert!(refund["message"].as_str().is_some());
}

#[tokio::test]
async fn test_paypal_status() {
    let client = setup_authenticated_client().await;

    let resp = client.get_http_client().post(&format!("{}/api/v1/paypal/status", get_api_base_url()))
        .header("Authorization", format!("Bearer {}", client.get_auth_token().unwrap()))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let status: serde_json::Value = resp.json().await.unwrap();

    // Status should return configuration info
    assert!(status["configured"].is_boolean());
    assert!(status["available_gateways"].is_array());
}

#[tokio::test]
async fn test_paypal_refund_with_partial_amount() {
    let client = setup_authenticated_client().await;

    // Create an order
    let create_resp = client.get_http_client().post(&format!("{}/api/v1/paypal/create-order", get_api_base_url()))
        .header("Authorization", format!("Bearer {}", client.get_auth_token().unwrap()))
        .json(&json!({
            "amount": 200.00,
            "currency": "USD"
        }))
        .send()
        .await
        .unwrap();

    let order: serde_json::Value = create_resp.json().await.unwrap();
    let order_id = order["order_id"].as_str().unwrap();

    // Partial refund
    let refund_resp = client.get_http_client().post(&format!("{}/api/v1/paypal/refund", get_api_base_url()))
        .header("Authorization", format!("Bearer {}", client.get_auth_token().unwrap()))
        .json(&json!({
            "payment_id": order_id,
            "amount": 150.00,
            "reason": "Partial refund"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(refund_resp.status(), 200);
    let refund: serde_json::Value = refund_resp.json().await.unwrap();
    assert_eq!(refund["amount"], 150.00);
}
