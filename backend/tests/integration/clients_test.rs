use crate::integration::{test_client::ApiTestClient, utils::get_api_base_url};
use serde_json::Value;

async fn setup_authenticated_client() -> ApiTestClient {
    let base_url = get_api_base_url();
    let client = ApiTestClient::new(base_url);

    let unique_id = crate::integration::utils::get_unique_id();
    let email = format!("client_test_{}@example.com", unique_id);
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
async fn test_client_crud() {
    let client = setup_authenticated_client().await;

    // 1. Create client
    let resp = client.create_client("John Doe", "john@example.com").await.unwrap();
    assert_eq!(resp.status(), 201, "Create should return 201");
    let created: Value = resp.json().await.unwrap();
    let client_id = created["id"].as_str().unwrap().to_string();
    assert_eq!(created["name"], "John Doe");
    assert_eq!(created["email"], "john@example.com");

    // 2. Get client
    let resp = client.get_client(&client_id).await.unwrap();
    assert_eq!(resp.status(), 200);
    let fetched: Value = resp.json().await.unwrap();
    assert_eq!(fetched["id"], client_id);

    // 3. List clients
    let resp = client.list_clients().await.unwrap();
    assert_eq!(resp.status(), 200);
    let list: Value = resp.json().await.unwrap();
    assert!(list.is_array());
    assert!(list.as_array().unwrap().len() > 0);

    // 4. Update client
    let resp = client.update_client(&client_id, "Jane Doe", "jane@example.com").await.unwrap();
    assert_eq!(resp.status(), 200);
    let updated: Value = resp.json().await.unwrap();
    assert_eq!(updated["name"], "Jane Doe");
    assert_eq!(updated["email"], "jane@example.com");

    // 5. Get client stats
    let resp = client.get_client_stats().await.unwrap();
    assert_eq!(resp.status(), 200);
    let stats: Value = resp.json().await.unwrap();
    assert!(stats["total_clients"].is_number());

    // 6. Delete client
    let resp = client.delete_client(&client_id).await.unwrap();
    assert_eq!(resp.status(), 204);

    // 7. Verify deletion
    let resp = client.get_client(&client_id).await.unwrap();
    assert_eq!(resp.status(), 404, "Should return 404 after deletion");
}

#[tokio::test]
async fn test_client_with_invoices() {
    let client = setup_authenticated_client().await;

    // Create client
    let resp = client.create_client("Invoice Client", "invoice@example.com").await.unwrap();
    let created: Value = resp.json().await.unwrap();
    let client_id = created["id"].as_str().unwrap().to_string();

    // Create invoice for client
    let resp = client.create_invoice(&client_id, 100.0).await.unwrap();
    assert_eq!(resp.status(), 201);
    let invoice: Value = resp.json().await.unwrap();
    let invoice_id = invoice["id"].as_str().unwrap().to_string();

    // Get client invoices
    let resp = client.get_client_invoices(&client_id).await.unwrap();
    assert_eq!(resp.status(), 200);
    let invoices: Value = resp.json().await.unwrap();
    assert!(invoices.is_array());
    assert_eq!(invoices.as_array().unwrap().len(), 1);

    // Cleanup
    client.delete_invoice(&invoice_id).await.unwrap();
    client.delete_client(&client_id).await.unwrap();
}

#[tokio::test]
async fn test_client_not_found() {
    let client = setup_authenticated_client().await;

    let resp = client.get_client("00000000-0000-0000-0000-000000000000").await.unwrap();
    assert_eq!(resp.status(), 404, "Non-existent client should return 404");
}
