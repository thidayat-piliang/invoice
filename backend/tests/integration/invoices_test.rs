use crate::integration::{test_client::ApiTestClient, utils::get_api_base_url};
use serde_json::Value;

async fn setup_authenticated_client() -> ApiTestClient {
    let base_url = get_api_base_url();
    let client = ApiTestClient::new(base_url);

    let unique_id = chrono::Utc::now().timestamp();
    let email = format!("invoice_test_{}@example.com", unique_id);
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
async fn test_invoice_crud() {
    let client = setup_authenticated_client().await;

    // Create client first
    let resp = client.create_client("Invoice Test Client", "invoice@test.com").await.unwrap();
    let client_data: Value = resp.json().await.unwrap();
    let client_id = client_data["id"].as_str().unwrap().to_string();

    // 1. Create invoice
    let resp = client.create_invoice(&client_id, 500.0).await.unwrap();
    assert_eq!(resp.status(), 201);
    let invoice: Value = resp.json().await.unwrap();
    let invoice_id = invoice["id"].as_str().unwrap().to_string();
    assert_eq!(invoice["total_amount"], 500.0);
    assert_eq!(invoice["status"], "draft");

    // 2. Get invoice
    let resp = client.get_invoice(&invoice_id).await.unwrap();
    assert_eq!(resp.status(), 200);
    let fetched: Value = resp.json().await.unwrap();
    assert_eq!(fetched["id"], invoice_id);

    // 3. List invoices
    let resp = client.list_invoices().await.unwrap();
    assert_eq!(resp.status(), 200);
    let list: Value = resp.json().await.unwrap();
    assert!(list.is_array());

    // 4. Update invoice
    let resp = client.update_invoice(&invoice_id, "Updated notes").await.unwrap();
    assert_eq!(resp.status(), 200);
    let updated: Value = resp.json().await.unwrap();
    assert_eq!(updated["notes"], "Updated notes");

    // 5. Send invoice
    let resp = client.send_invoice(&invoice_id).await.unwrap();
    assert_eq!(resp.status(), 200);

    // 6. Get PDF
    let resp = client.get_invoice_pdf(&invoice_id).await.unwrap();
    assert_eq!(resp.status(), 200);

    // 7. Record payment
    let resp = client.record_payment(&invoice_id, 250.0).await.unwrap();
    assert_eq!(resp.status(), 201);

    // 8. Send reminder
    let resp = client.send_reminder(&invoice_id).await.unwrap();
    assert_eq!(resp.status(), 200);

    // 9. Delete invoice
    let resp = client.delete_invoice(&invoice_id).await.unwrap();
    assert_eq!(resp.status(), 204);

    // Cleanup
    client.delete_client(&client_id).await.unwrap();
}

#[tokio::test]
async fn test_invoice_with_multiple_items() {
    let client = setup_authenticated_client().await;

    // Create client
    let resp = client.create_client("Multi Item Client", "multi@test.com").await.unwrap();
    let client_data: Value = resp.json().await.unwrap();
    let client_id = client_data["id"].as_str().unwrap().to_string();

    // Create invoice with custom items
    let mut request = client.clone();
    let resp = request.client.post(&format!("{}/api/v1/invoices", get_api_base_url()))
        .header("Authorization", format!("Bearer {}", request.auth_token.unwrap()))
        .json(&serde_json::json!({
            "client_id": client_id,
            "items": [
                {
                    "description": "Service A",
                    "quantity": 2,
                    "unit_price": 100.0,
                    "tax_rate": 10.0
                },
                {
                    "description": "Service B",
                    "quantity": 1,
                    "unit_price": 200.0,
                    "tax_rate": 0.0
                }
            ],
            "notes": "Multi-item invoice",
            "terms": "Net 30"
        }))
        .send()
        .await.unwrap();

    assert_eq!(resp.status(), 201);
    let invoice: Value = resp.json().await.unwrap();

    // Should have subtotal 400 (2*100 + 200) + tax 20 (10% of 200) = 420
    assert_eq!(invoice["total_amount"], 420.0);

    // Cleanup
    let invoice_id = invoice["id"].as_str().unwrap().to_string();
    client.delete_invoice(&invoice_id).await.unwrap();
    client.delete_client(&client_id).await.unwrap();
}

#[tokio::test]
async fn test_invoice_status_transitions() {
    let client = setup_authenticated_client().await;

    // Create client and invoice
    let resp = client.create_client("Status Test", "status@test.com").await.unwrap();
    let client_data: Value = resp.json().await.unwrap();
    let client_id = client_data["id"].as_str().unwrap().to_string();

    let resp = client.create_invoice(&client_id, 100.0).await.unwrap();
    let invoice: Value = resp.json().await.unwrap();
    let invoice_id = invoice["id"].as_str().unwrap().to_string();

    // Initial status should be draft
    assert_eq!(invoice["status"], "draft");

    // Send invoice (should change status)
    client.send_invoice(&invoice_id).await.unwrap();
    let resp = client.get_invoice(&invoice_id).await.unwrap();
    let sent_invoice: Value = resp.json().await.unwrap();
    // Status might be "sent" or "pending" depending on implementation
    assert_ne!(sent_invoice["status"], "draft");

    // Record full payment
    client.record_payment(&invoice_id, 100.0).await.unwrap();
    let resp = client.get_invoice(&invoice_id).await.unwrap();
    let paid_invoice: Value = resp.json().await.unwrap();
    assert_eq!(paid_invoice["status"], "paid");

    // Cleanup
    client.delete_invoice(&invoice_id).await.unwrap();
    client.delete_client(&client_id).await.unwrap();
}
