use crate::integration::{test_client::ApiTestClient, utils::get_api_base_url};
use serde_json::Value;

#[tokio::test]
async fn test_full_auth_flow() {
    let base_url = get_api_base_url();
    let client = ApiTestClient::new(base_url);

    // Test health check first
    let health = client.health_check().await.unwrap();
    assert_eq!(health, "OK");

    // Generate unique email for this test
    let unique_id = crate::integration::utils::get_unique_id();
    let email = format!("test_user_{}@example.com", unique_id);
    let password = "testpassword123";

    // 1. Register
    let resp = client.register(&email, password, Some("Test Company")).await.unwrap();
    assert_eq!(resp.status(), 201, "Register should return 201");
    let register_data: Value = resp.json().await.unwrap();
    assert!(register_data["access_token"].is_string(), "Should return access token");

    // 2. Login
    let resp = client.login(&email, password).await.unwrap();
    assert_eq!(resp.status(), 200, "Login should return 200");
    let login_data: Value = resp.json().await.unwrap();
    let token = login_data["access_token"].as_str().unwrap().to_string();
    assert!(!token.is_empty(), "Token should not be empty");

    // 3. Create new client with token
    let mut authed_client = client.clone();
    authed_client.set_token(token);

    // 4. Get current user
    let resp = authed_client.get_current_user().await.unwrap();
    assert_eq!(resp.status(), 200, "Get current user should return 200");
    let user_data: Value = resp.json().await.unwrap();
    assert_eq!(user_data["email"], email, "Email should match");
    assert_eq!(user_data["company_name"], "Test Company", "Company name should match");

    // 5. Update profile
    let resp = authed_client.update_profile("+1234567890", "Updated Company").await.unwrap();
    assert_eq!(resp.status(), 200, "Update profile should return 200");
    let updated_data: Value = resp.json().await.unwrap();
    assert_eq!(updated_data["phone"], "+1234567890", "Phone should be updated");
    assert_eq!(updated_data["company_name"], "Updated Company", "Company name should be updated");

    // 6. Test invalid login
    let resp = client.login(&email, "wrongpassword").await.unwrap();
    assert_eq!(resp.status(), 401, "Wrong password should return 401");

    // 7. Test register with existing email
    let resp = client.register(&email, password, None).await.unwrap();
    assert_eq!(resp.status(), 400, "Duplicate email should return 400");
}

#[tokio::test]
async fn test_auth_without_token() {
    let base_url = get_api_base_url();
    let client = ApiTestClient::new(base_url);

    // Try to access protected endpoint without token
    let resp = client.get_current_user().await.unwrap();
    assert_eq!(resp.status(), 401, "Should return 401 without token");

    // Try to create client without token
    let resp = client.create_client("Test", "test@example.com").await.unwrap();
    assert_eq!(resp.status(), 401, "Should return 401 without token");
}

#[tokio::test]
async fn test_register_validation() {
    let base_url = get_api_base_url();
    let client = ApiTestClient::new(base_url);

    // Test invalid email
    let resp = client.register("invalid-email", "password123", None).await.unwrap();
    assert_eq!(resp.status(), 400, "Invalid email should return 400");

    // Test short password
    let resp = client.register("test@example.com", "short", None).await.unwrap();
    assert_eq!(resp.status(), 400, "Short password should return 400");
}
