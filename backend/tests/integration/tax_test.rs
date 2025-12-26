use crate::integration::{test_client::ApiTestClient, utils::get_api_base_url};
use serde_json::Value;

async fn setup_authenticated_client() -> ApiTestClient {
    let base_url = get_api_base_url();
    let client = ApiTestClient::new(base_url);

    let unique_id = crate::integration::utils::get_unique_id();
    let email = format!("tax_test_{}@example.com", unique_id);
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
async fn test_tax_settings_crud() {
    let client = setup_authenticated_client().await;

    // 1. Create tax setting
    let resp = client.get_http_client().post(&format!("{}/api/v1/settings/tax", get_api_base_url()))
        .header("Authorization", format!("Bearer {}", client.get_auth_token().unwrap()))
        .json(&serde_json::json!({
            "label": "Sales Tax",
            "rate": 0.0825,
            "is_default": true
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let tax_setting: Value = resp.json().await.unwrap();
    assert_eq!(tax_setting["label"], "Sales Tax");
    assert_eq!(tax_setting["rate"], 0.0825);
    assert_eq!(tax_setting["is_default"], true);
    let tax_id = tax_setting["id"].as_str().unwrap().to_string();

    // 2. Get all tax settings
    let resp = client.get_http_client().get(&format!("{}/api/v1/settings/tax", get_api_base_url()))
        .header("Authorization", format!("Bearer {}", client.get_auth_token().unwrap()))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let settings: Value = resp.json().await.unwrap();
    assert!(settings.is_array());
    assert_eq!(settings.as_array().unwrap().len(), 1);

    // 3. Get default tax
    let resp = client.get_http_client().get(&format!("{}/api/v1/settings/tax/default", get_api_base_url()))
        .header("Authorization", format!("Bearer {}", client.get_auth_token().unwrap()))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let default: Value = resp.json().await.unwrap();
    assert_eq!(default["label"], "Sales Tax");

    // 4. Update tax setting
    let resp = client.get_http_client().put(&format!("{}/api/v1/settings/tax/{}", get_api_base_url(), tax_id))
        .header("Authorization", format!("Bearer {}", client.get_auth_token().unwrap()))
        .json(&serde_json::json!({
            "label": "Updated Tax",
            "rate": 0.10
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let updated: Value = resp.json().await.unwrap();
    assert_eq!(updated["label"], "Updated Tax");
    assert_eq!(updated["rate"], 0.10);

    // 5. Calculate tax
    let resp = client.get_http_client().post(&format!("{}/api/v1/tax/calculate", get_api_base_url()))
        .header("Authorization", format!("Bearer {}", client.get_auth_token().unwrap()))
        .json(&serde_json::json!({
            "amount": 100.0
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let calc: Value = resp.json().await.unwrap();
    assert_eq!(calc["calculation"]["subtotal"], 100.0);
    assert_eq!(calc["calculation"]["tax_rate"], 0.10);
    assert_eq!(calc["calculation"]["tax_amount"], 10.0);
    assert_eq!(calc["calculation"]["total"], 110.0);
    assert!(calc["legal_disclaimer"].as_str().unwrap().contains("does not calculate"));
}

#[tokio::test]
async fn test_invoice_with_tax() {
    let client = setup_authenticated_client().await;

    // Create tax setting first
    let resp = client.get_http_client().post(&format!("{}/api/v1/settings/tax", get_api_base_url()))
        .header("Authorization", format!("Bearer {}", client.get_auth_token().unwrap()))
        .json(&serde_json::json!({
            "label": "VAT",
            "rate": 0.20,
            "is_default": true
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // Create client
    let resp = client.create_client("Tax Client", "taxclient@test.com").await.unwrap();
    let client_data: Value = resp.json().await.unwrap();
    let client_id = client_data["id"].as_str().unwrap().to_string();

    // Create invoice (should automatically apply default tax)
    let today = chrono::Utc::now().naive_utc().date();
    let due_date = today + chrono::Duration::days(30);

    let resp = client.get_http_client().post(&format!("{}/api/v1/invoices", get_api_base_url()))
        .header("Authorization", format!("Bearer {}", client.get_auth_token().unwrap()))
        .json(&serde_json::json!({
            "client_id": client_id,
            "issue_date": today,
            "due_date": due_date,
            "items": [
                {
                    "description": "Taxable Service",
                    "quantity": 2,
                    "unit_price": 50.0
                    // Note: no tax_rate specified, should use default
                }
            ],
            "notes": "Invoice with tax",
            "tax_included": false,
            "send_immediately": false
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 201);
    let invoice: Value = resp.json().await.unwrap();

    // Verify tax was applied
    assert_eq!(invoice["subtotal"], 100.0); // 2 * 50
    assert_eq!(invoice["tax_amount"], 20.0); // 100 * 0.20
    assert_eq!(invoice["total_amount"], 120.0); // 100 + 20
    assert_eq!(invoice["tax_label"], "VAT");

    // Get invoice detail to verify tax_id is present
    let invoice_id = invoice["id"].as_str().unwrap();
    let resp = client.get_http_client().get(&format!("{}/api/v1/invoices/{}", get_api_base_url(), invoice_id))
        .header("Authorization", format!("Bearer {}", client.get_auth_token().unwrap()))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let detail: Value = resp.json().await.unwrap();
    assert!(detail["tax_id"].as_str().is_some());
    assert_eq!(detail["tax_label"], "VAT");
}

#[tokio::test]
async fn test_tax_summary() {
    let client = setup_authenticated_client().await;

    // Create tax setting
    let resp = client.get_http_client().post(&format!("{}/api/v1/settings/tax", get_api_base_url()))
        .header("Authorization", format!("Bearer {}", client.get_auth_token().unwrap()))
        .json(&serde_json::json!({
            "label": "Sales Tax",
            "rate": 0.05,
            "is_default": true
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // Create client
    let resp = client.create_client("Summary Client", "summary@test.com").await.unwrap();
    let client_data: Value = resp.json().await.unwrap();
    let client_id = client_data["id"].as_str().unwrap().to_string();

    // Create multiple invoices
    let today = chrono::Utc::now().naive_utc().date();
    let due_date = today + chrono::Duration::days(30);

    for i in 1..=3 {
        client.get_http_client().post(&format!("{}/api/v1/invoices", get_api_base_url()))
            .header("Authorization", format!("Bearer {}", client.get_auth_token().unwrap()))
            .json(&serde_json::json!({
                "client_id": client_id,
                "issue_date": today,
                "due_date": due_date,
                "items": [{
                    "description": format!("Item {}", i),
                    "quantity": 1,
                    "unit_price": 100.0
                }],
                "notes": "Test",
                "tax_included": false,
                "send_immediately": false
            }))
            .send()
            .await
            .unwrap();
    }

    // Get tax summary
    let start_date = today.format("%Y-%m-%d").to_string();
    let end_date = today.format("%Y-%m-%d").to_string();

    let resp = client.get_http_client().post(&format!("{}/api/v1/tax/summary", get_api_base_url()))
        .header("Authorization", format!("Bearer {}", client.get_auth_token().unwrap()))
        .json(&serde_json::json!({
            "start_date": start_date,
            "end_date": end_date
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let summary: Value = resp.json().await.unwrap();

    // 3 invoices * 100 = 300 subtotal
    // 300 * 0.05 = 15 tax
    // 300 + 15 = 315 total
    assert_eq!(summary["subtotal"], 300.0);
    assert_eq!(summary["tax_collected"], 15.0);
    assert_eq!(summary["total"], 315.0);
}

#[tokio::test]
async fn test_validate_tax_id() {
    let client = setup_authenticated_client().await;

    // Test valid EIN format
    let resp = client.get_http_client().post(&format!("{}/api/v1/tax/validate", get_api_base_url()))
        .header("Authorization", format!("Bearer {}", client.get_auth_token().unwrap()))
        .json(&serde_json::json!({
            "tax_id": "12-3456789"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let result: Value = resp.json().await.unwrap();
    assert_eq!(result["is_valid"], true);
    assert_eq!(result["normalized"], "12-3456789");

    // Test invalid format
    let resp = client.get_http_client().post(&format!("{}/api/v1/tax/validate", get_api_base_url()))
        .header("Authorization", format!("Bearer {}", client.get_auth_token().unwrap()))
        .json(&serde_json::json!({
            "tax_id": "INVALID"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let result: Value = resp.json().await.unwrap();
    assert_eq!(result["is_valid"], false);
}

#[tokio::test]
async fn test_delete_tax_setting() {
    let client = setup_authenticated_client().await;

    // Create two tax settings
    let resp1 = client.get_http_client().post(&format!("{}/api/v1/settings/tax", get_api_base_url()))
        .header("Authorization", format!("Bearer {}", client.get_auth_token().unwrap()))
        .json(&serde_json::json!({
            "label": "Tax 1",
            "rate": 0.05,
            "is_default": true
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp1.status(), 200);
    let tax1: Value = resp1.json().await.unwrap();
    let tax1_id = tax1["id"].as_str().unwrap();

    let resp2 = client.get_http_client().post(&format!("{}/api/v1/settings/tax", get_api_base_url()))
        .header("Authorization", format!("Bearer {}", client.get_auth_token().unwrap()))
        .json(&serde_json::json!({
            "label": "Tax 2",
            "rate": 0.10,
            "is_default": false
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp2.status(), 200);

    // Delete first tax setting
    let resp = client.get_http_client().delete(&format!("{}/api/v1/settings/tax/{}", get_api_base_url(), tax1_id))
        .header("Authorization", format!("Bearer {}", client.get_auth_token().unwrap()))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 204);

    // Verify it's deleted
    let settings_resp = client.get_http_client().get(&format!("{}/api/v1/settings/tax", get_api_base_url()))
        .header("Authorization", format!("Bearer {}", client.get_auth_token().unwrap()))
        .send()
        .await
        .unwrap();

    let settings: Value = settings_resp.json().await.unwrap();
    assert_eq!(settings.as_array().unwrap().len(), 1);
}
