use crate::integration::{test_client::ApiTestClient, utils::get_api_base_url};
use serde_json::Value;

async fn setup_authenticated_client() -> ApiTestClient {
    let base_url = get_api_base_url();
    let client = ApiTestClient::new(base_url);

    let unique_id = crate::integration::utils::get_unique_id();
    let email = format!("discussion_test_{}@example.com", unique_id);
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
async fn test_discussion_seller_add_message() {
    let client = setup_authenticated_client().await;

    // Create client
    let resp = client.create_client("Discussion Client", "discussion@test.com").await.unwrap();
    let client_data: Value = resp.json().await.unwrap();
    let client_id = client_data["id"].as_str().unwrap().to_string();

    // Create invoice
    let resp = client.create_invoice(&client_id, 100.0).await.unwrap();
    let invoice: Value = resp.json().await.unwrap();
    let invoice_id = invoice["id"].as_str().unwrap().to_string();

    // Add discussion message
    let resp = client.add_discussion_message(&invoice_id, "This is a test message from seller").await.unwrap();
    assert_eq!(resp.status(), 201);

    let message: Value = resp.json().await.unwrap();
    assert_eq!(message["message"], "This is a test message from seller");
    assert_eq!(message["sender_type"], "seller");

    // Get discussion messages
    let resp = client.get_discussion_messages(&invoice_id).await.unwrap();
    assert_eq!(resp.status(), 200);

    let messages: Value = resp.json().await.unwrap();
    assert!(messages["messages"].is_array());
    assert_eq!(messages["messages"].as_array().unwrap().len(), 1);

    // Cleanup
    client.delete_invoice(&invoice_id).await.unwrap();
    client.delete_client(&client_id).await.unwrap();
}

#[tokio::test]
async fn test_discussion_multiple_messages() {
    let client = setup_authenticated_client().await;

    // Create client and invoice
    let resp = client.create_client("Multi Message Client", "multi@test.com").await.unwrap();
    let client_data: Value = resp.json().await.unwrap();
    let client_id = client_data["id"].as_str().unwrap().to_string();

    let resp = client.create_invoice(&client_id, 200.0).await.unwrap();
    let invoice: Value = resp.json().await.unwrap();
    let invoice_id = invoice["id"].as_str().unwrap().to_string();

    // Add multiple messages
    client.add_discussion_message(&invoice_id, "First message").await.unwrap();
    client.add_discussion_message(&invoice_id, "Second message").await.unwrap();
    client.add_discussion_message(&invoice_id, "Third message").await.unwrap();

    // Get all messages
    let resp = client.get_discussion_messages(&invoice_id).await.unwrap();
    let messages: Value = resp.json().await.unwrap();
    let messages_array = messages["messages"].as_array().unwrap();

    assert_eq!(messages_array.len(), 3);

    // Verify order
    assert_eq!(messages_array[0]["message"], "First message");
    assert_eq!(messages_array[1]["message"], "Second message");
    assert_eq!(messages_array[2]["message"], "Third message");

    // Cleanup
    client.delete_invoice(&invoice_id).await.unwrap();
    client.delete_client(&client_id).await.unwrap();
}

#[tokio::test]
async fn test_discussion_guest_access() {
    let client = setup_authenticated_client().await;

    // Create client and invoice
    let resp = client.create_client("Guest Discussion Client", "guest@test.com").await.unwrap();
    let client_data: Value = resp.json().await.unwrap();
    let client_id = client_data["id"].as_str().unwrap().to_string();

    let resp = client.create_invoice(&client_id, 150.0).await.unwrap();
    let created: Value = resp.json().await.unwrap();
    let invoice_id = created["id"].as_str().unwrap().to_string();

    // Get full invoice details to get guest_payment_token
    let resp = client.get_invoice(&invoice_id).await.unwrap();
    let invoice: Value = resp.json().await.unwrap();
    let guest_token = invoice["guest_payment_token"].as_str().unwrap().to_string();

    // Seller adds a message
    client.add_discussion_message(&invoice_id, "Hello from seller").await.unwrap();

    // Guest adds a message
    let resp = client.add_guest_discussion_message(&guest_token, "Hello from buyer").await.unwrap();
    assert_eq!(resp.status(), 201);

    let message: Value = resp.json().await.unwrap();
    assert_eq!(message["message"], "Hello from buyer");
    assert_eq!(message["sender_type"], "buyer");

    // Guest gets messages
    let resp = client.get_guest_discussion_messages(&guest_token).await.unwrap();
    assert_eq!(resp.status(), 200);

    let messages: Value = resp.json().await.unwrap();
    let messages_array = messages["messages"].as_array().unwrap();

    assert_eq!(messages_array.len(), 2);

    // Cleanup
    client.delete_invoice(&invoice_id).await.unwrap();
    client.delete_client(&client_id).await.unwrap();
}

#[tokio::test]
async fn test_discussion_invalid_invoice() {
    let client = setup_authenticated_client().await;

    // Try to get messages for non-existent invoice
    let fake_id = "00000000-0000-0000-0000-000000000000";
    let resp = client.get_discussion_messages(fake_id).await.unwrap();

    // Should return error (404 or similar)
    assert!(resp.status().as_u16() >= 400);
}

#[tokio::test]
async fn test_discussion_guest_invalid_token() {
    let client = ApiTestClient::new(get_api_base_url());

    // Try to add message with invalid token
    let resp = client.add_guest_discussion_message("invalid_token", "Test message").await.unwrap();

    // Should return error
    assert!(resp.status().as_u16() >= 400);
}

#[tokio::test]
async fn test_discussion_empty_messages() {
    let client = setup_authenticated_client().await;

    // Create client and invoice
    let resp = client.create_client("Empty Discussion Client", "empty@test.com").await.unwrap();
    let client_data: Value = resp.json().await.unwrap();
    let client_id = client_data["id"].as_str().unwrap().to_string();

    let resp = client.create_invoice(&client_id, 100.0).await.unwrap();
    let invoice: Value = resp.json().await.unwrap();
    let invoice_id = invoice["id"].as_str().unwrap().to_string();

    // Get messages when none exist
    let resp = client.get_discussion_messages(&invoice_id).await.unwrap();
    assert_eq!(resp.status(), 200);

    let messages: Value = resp.json().await.unwrap();
    assert!(messages["messages"].is_array());
    assert_eq!(messages["messages"].as_array().unwrap().len(), 0);

    // Cleanup
    client.delete_invoice(&invoice_id).await.unwrap();
    client.delete_client(&client_id).await.unwrap();
}
