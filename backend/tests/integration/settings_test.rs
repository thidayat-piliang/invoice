use crate::integration::{test_client::ApiTestClient, utils::get_api_base_url};
use serde_json::Value;

async fn setup_authenticated_client() -> ApiTestClient {
    let base_url = get_api_base_url();
    let client = ApiTestClient::new(base_url);

    let unique_id = chrono::Utc::now().timestamp();
    let email = format!("settings_test_{}@example.com", unique_id);
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
async fn test_business_settings() {
    let client = setup_authenticated_client().await;

    // Get initial settings
    let resp = client.get_business_settings().await.unwrap();
    assert_eq!(resp.status(), 200);
    let initial: Value = resp.json().await.unwrap();
    assert!(initial["email"].is_string());

    // Update settings
    let resp = client.update_business_settings("Updated Co", "+9999999999").await.unwrap();
    assert_eq!(resp.status(), 200);
    let updated: Value = resp.json().await.unwrap();
    assert_eq!(updated["company_name"], "Updated Co");
    assert_eq!(updated["phone"], "+9999999999");

    // Verify update persisted
    let resp = client.get_business_settings().await.unwrap();
    let verify: Value = resp.json().await.unwrap();
    assert_eq!(verify["company_name"], "Updated Co");
}

#[tokio::test]
async fn test_tax_settings() {
    let client = setup_authenticated_client().await;

    // Get initial tax settings
    let resp = client.get_tax_settings().await.unwrap();
    assert_eq!(resp.status(), 200);
    let initial: Value = resp.json().await.unwrap();
    assert!(initial["tax_settings"].is_object());
    assert!(initial["available_states"].is_array());

    // Update tax settings
    let resp = client.update_tax_settings("CA", 7.5).await.unwrap();
    assert_eq!(resp.status(), 200);
    let updated: Value = resp.json().await.unwrap();
    assert_eq!(updated["state_code"], "CA");
    assert_eq!(updated["tax_rate"], 7.5);

    // Verify update
    let resp = client.get_tax_settings().await.unwrap();
    let verify: Value = resp.json().await.unwrap();
    assert_eq!(verify["tax_settings"]["state_code"], "CA");
}

#[tokio::test]
async fn test_notification_settings() {
    let client = setup_authenticated_client().await;

    // Get initial settings
    let resp = client.get_notification_settings().await.unwrap();
    assert_eq!(resp.status(), 200);
    let initial: Value = resp.json().await.unwrap();
    assert!(initial["email_payment_received"].is_boolean());

    // Update settings
    let resp = client.update_notification_settings().await.unwrap();
    assert_eq!(resp.status(), 200);
    let updated: Value = resp.json().await.unwrap();
    assert_eq!(updated["email_payment_received"], true);

    // Verify update
    let resp = client.get_notification_settings().await.unwrap();
    let verify: Value = resp.json().await.unwrap();
    assert_eq!(verify["email_payment_received"], true);
}

#[tokio::test]
async fn test_invoice_settings() {
    let client = setup_authenticated_client().await;

    // Get initial invoice settings
    let resp = client.get_invoice_settings().await.unwrap();
    assert_eq!(resp.status(), 200);
    let initial: Value = resp.json().await.unwrap();
    assert!(initial["template"].is_string());
    assert!(initial["terms"].is_string());

    // Update invoice settings
    let resp = client.update_invoice_settings("premium", "Net 15", "Thank you for your business!").await.unwrap();
    assert_eq!(resp.status(), 200);
    let updated: Value = resp.json().await.unwrap();
    assert_eq!(updated["template"], "premium");
    assert_eq!(updated["terms"], "Net 15");
    assert_eq!(updated["notes"], "Thank you for your business!");

    // Verify update
    let resp = client.get_invoice_settings().await.unwrap();
    let verify: Value = resp.json().await.unwrap();
    assert_eq!(verify["template"], "premium");
    assert_eq!(verify["terms"], "Net 15");
}

#[tokio::test]
async fn test_all_settings_endpoints() {
    let client = setup_authenticated_client().await;

    // Test all settings endpoints return valid responses
    let endpoints = [
        ("business", || client.get_business_settings()),
        ("tax", || client.get_tax_settings()),
        ("notifications", || client.get_notification_settings()),
        ("invoice", || client.get_invoice_settings()),
    ];

    for (name, get_fn) in endpoints {
        let resp = get_fn().await.unwrap();
        assert_eq!(resp.status(), 200, "{} settings should return 200", name);
        let data: Value = resp.json().await.unwrap();
        assert!(data.is_object(), "{} settings should return object", name);
    }
}

#[tokio::test]
async fn test_settings_persistence() {
    let client = setup_authenticated_client().await;

    // Update multiple settings
    client.update_business_settings("Persistent Co", "+1111111111").await.unwrap();
    client.update_tax_settings("TX", 6.25).await.unwrap();
    client.update_invoice_settings("classic", "Net 30", "Please pay on time").await.unwrap();

    // Verify all settings are persisted
    let business = client.get_business_settings().await.unwrap().json::<Value>().await.unwrap();
    let tax = client.get_tax_settings().await.unwrap().json::<Value>().await.unwrap();
    let invoice = client.get_invoice_settings().await.unwrap().json::<Value>().await.unwrap();

    assert_eq!(business["company_name"], "Persistent Co");
    assert_eq!(tax["tax_settings"]["state_code"], "TX");
    assert_eq!(invoice["terms"], "Net 30");
}
