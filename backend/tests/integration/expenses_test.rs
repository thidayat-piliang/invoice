use crate::integration::{test_client::ApiTestClient, utils::get_api_base_url};
use serde_json::Value;

async fn setup_authenticated_client() -> ApiTestClient {
    let base_url = get_api_base_url();
    let client = ApiTestClient::new(base_url);

    let unique_id = crate::integration::utils::get_unique_id();
    let email = format!("expense_test_{}@example.com", unique_id);
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
async fn test_expense_crud() {
    let client = setup_authenticated_client().await;

    // 1. Create expense
    let resp = client.create_expense(150.0, "office_supplies", "Office Depot").await.unwrap();
    assert_eq!(resp.status(), 201);
    let expense: Value = resp.json().await.unwrap();
    let expense_id = expense["id"].as_str().unwrap().to_string();
    assert_eq!(expense["amount"], 150.0);
    assert_eq!(expense["category"], "office_supplies");
    assert_eq!(expense["vendor"], "Office Depot");

    // 2. Get expense
    let resp = client.get_expense(&expense_id).await.unwrap();
    assert_eq!(resp.status(), 200);
    let fetched: Value = resp.json().await.unwrap();
    assert_eq!(fetched["id"], expense_id);

    // 3. List expenses
    let resp = client.list_expenses().await.unwrap();
    assert_eq!(resp.status(), 200);
    let list: Value = resp.json().await.unwrap();
    assert!(list.is_array());
    assert!(list.as_array().unwrap().len() > 0);

    // 4. Update expense
    let resp = client.update_expense(&expense_id, "Updated description").await.unwrap();
    assert_eq!(resp.status(), 200);
    let updated: Value = resp.json().await.unwrap();
    assert_eq!(updated["description"], "Updated description");

    // 5. Get expense stats
    let resp = client.get_expense_stats().await.unwrap();
    assert_eq!(resp.status(), 200);
    let stats: Value = resp.json().await.unwrap();
    assert!(stats["total_expenses"].is_number());

    // 6. Delete expense
    let resp = client.delete_expense(&expense_id).await.unwrap();
    assert_eq!(resp.status(), 204);

    // 7. Verify deletion
    let resp = client.get_expense(&expense_id).await.unwrap();
    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn test_expense_with_filters() {
    let client = setup_authenticated_client().await;

    // Create multiple expenses with different categories
    client.create_expense(100.0, "travel", "Airline").await.unwrap();
    client.create_expense(200.0, "office_supplies", "Staples").await.unwrap();
    client.create_expense(150.0, "travel", "Hotel").await.unwrap();

    // List all expenses
    let resp = client.list_expenses().await.unwrap();
    let all_expenses: Value = resp.json().await.unwrap();
    assert_eq!(all_expenses.as_array().unwrap().len(), 3);

    // Cleanup - delete all
    for expense in all_expenses.as_array().unwrap() {
        let id = expense["id"].as_str().unwrap();
        client.delete_expense(id).await.unwrap();
    }
}

#[tokio::test]
async fn test_expense_tax_deductible() {
    let client = setup_authenticated_client().await;

    // Create tax deductible expense
    let resp = client.create_expense(500.0, "equipment", "Amazon").await.unwrap();
    let expense: Value = resp.json().await.unwrap();
    assert_eq!(expense["tax_deductible"], true);

    // Create non-deductible expense
    let mut request = client.clone();
    let resp = request.get_http_client().post(&format!("{}/api/v1/expenses", get_api_base_url()))
        .header("Authorization", format!("Bearer {}", request.get_auth_token().unwrap()))
        .json(&serde_json::json!({
            "amount": 300.0,
            "category": "other",
            "vendor": "Restaurant",
            "description": "Team lunch",
            "date_incurred": "2025-01-01",
            "tax_deductible": false,
        }))
        .send()
        .await.unwrap();

    assert_eq!(resp.status(), 201);
    let expense2: Value = resp.json().await.unwrap();
    assert_eq!(expense2["tax_deductible"], false);

    // Cleanup
    client.delete_expense(expense["id"].as_str().unwrap()).await.unwrap();
    client.delete_expense(expense2["id"].as_str().unwrap()).await.unwrap();
}

#[tokio::test]
async fn test_expense_validation() {
    let client = setup_authenticated_client().await;

    // Missing required fields
    let mut request = client.clone();
    let resp = request.get_http_client().post(&format!("{}/api/v1/expenses", get_api_base_url()))
        .header("Authorization", format!("Bearer {}", request.get_auth_token().unwrap()))
        .json(&serde_json::json!({
            "amount": 100.0,
            // Missing category, vendor, date_incurred
        }))
        .send()
        .await.unwrap();

    // 422 is correct for JSON deserialization/validation errors
    assert!(resp.status() == 400 || resp.status() == 422, "Missing required fields should return 400 or 422, got {}", resp.status());
}
