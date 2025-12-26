use crate::integration::{test_client::ApiTestClient, utils::get_api_base_url};
use serde_json::Value;

async fn setup_authenticated_client_with_data() -> ApiTestClient {
    let base_url = get_api_base_url();
    let client = ApiTestClient::new(base_url);

    let unique_id = crate::integration::utils::get_unique_id();
    let email = format!("report_test_{}@example.com", unique_id);
    let password = "testpassword123";

    client.register(&email, password, Some("Report Company")).await.unwrap();
    let resp = client.login(&email, password).await.unwrap();
    let data: Value = resp.json().await.unwrap();
    let token = data["access_token"].as_str().unwrap().to_string();

    let mut authed_client = client.clone();
    authed_client.set_token(token);

    // Create some test data
    // Client
    let resp = authed_client.create_client("Report Client", "report@test.com").await.unwrap();
    let client_data: Value = resp.json().await.unwrap();
    let client_id = client_data["id"].as_str().unwrap().to_string();

    // Invoices
    authed_client.create_invoice(&client_id, 1000.0).await.unwrap();
    authed_client.create_invoice(&client_id, 2000.0).await.unwrap();

    // Expenses
    authed_client.create_expense(500.0, "office_supplies", "Staples").await.unwrap();
    authed_client.create_expense(300.0, "travel", "Airline").await.unwrap();

    // Payments
    let resp = authed_client.list_invoices().await.unwrap();
    let invoices: Value = resp.json().await.unwrap();
    if let Some(invoice) = invoices.as_array().and_then(|arr| arr.first()) {
        let invoice_id = invoice["id"].as_str().unwrap();
        authed_client.record_payment(invoice_id, 1000.0).await.unwrap();
    }

    authed_client
}

#[tokio::test]
async fn test_overview_stats() {
    let client = setup_authenticated_client_with_data().await;

    let resp = client.get_overview_stats().await.unwrap();
    assert_eq!(resp.status(), 200);
    let stats: Value = resp.json().await.unwrap();

    // Should have some stats
    assert!(stats["total_revenue"].is_number());
    assert!(stats["total_outstanding"].is_number());
    assert!(stats["paid_invoices"].is_number());
    assert!(stats["overdue_invoices"].is_number());
    assert!(stats["total_expenses"].is_number());
    assert!(stats["net_profit"].is_number());
}

#[tokio::test]
async fn test_income_report() {
    let client = setup_authenticated_client_with_data().await;

    let resp = client.get_income_report("2025-01-01", "2025-12-31").await.unwrap();
    assert_eq!(resp.status(), 200);
    let report: Value = resp.json().await.unwrap();

    assert!(report["total_income"].is_number());
    assert!(report["by_month"].is_array());
    assert!(report["by_client"].is_array());
}

#[tokio::test]
async fn test_expenses_report() {
    let client = setup_authenticated_client_with_data().await;

    let resp = client.get_expenses_report("2025-01-01", "2025-12-31").await.unwrap();
    assert_eq!(resp.status(), 200);
    let report: Value = resp.json().await.unwrap();

    assert!(report["total_expenses"].is_number());
    assert!(report["by_category"].is_array());
    assert!(report["by_month"].is_array());
}

#[tokio::test]
async fn test_tax_report() {
    let client = setup_authenticated_client_with_data().await;

    let resp = client.get_tax_report("2025-01-01", "2025-12-31").await.unwrap();
    assert_eq!(resp.status(), 200);
    let report: Value = resp.json().await.unwrap();

    assert!(report["total_tax_collected"].is_number());
    assert!(report["total_tax_deductible"].is_number());
    assert!(report["by_state"].is_array());
}

#[tokio::test]
async fn test_aging_report() {
    let client = setup_authenticated_client_with_data().await;

    let resp = client.get_aging_report().await.unwrap();
    assert_eq!(resp.status(), 200);
    let report: Value = resp.json().await.unwrap();

    assert!(report["current"].is_number());
    assert!(report["one_to_thirty_days"].is_number());
    assert!(report["thirty_one_to_sixty_days"].is_number());
    assert!(report["sixty_one_to_ninety_days"].is_number());
    assert!(report["over_ninety_days"].is_number());
}

#[tokio::test]
async fn test_export_report() {
    let client = setup_authenticated_client_with_data().await;

    // Export income report as CSV
    let resp = client.export_report("income", "csv", "2025-01-01", "2025-12-31").await.unwrap();
    assert_eq!(resp.status(), 200);
    let export: Value = resp.json().await.unwrap();

    assert!(export["download_url"].is_string());
    assert!(export["expires_at"].is_string());
}
