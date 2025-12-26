use reqwest::Client;
use std::time::Duration;

#[derive(Clone)]
pub struct ApiTestClient {
    base_url: String,
    client: Client,
    auth_token: Option<String>,
}

impl ApiTestClient {
    pub fn new(base_url: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            base_url,
            client,
            auth_token: None,
        }
    }

    pub fn set_token(&mut self, token: String) {
        self.auth_token = Some(token);
    }

    pub fn get_http_client(&self) -> &Client {
        &self.client
    }

    pub fn get_auth_token(&self) -> Option<&str> {
        self.auth_token.as_deref()
    }

    pub fn get_base_url(&self) -> &str {
        &self.base_url
    }

    fn get_auth_header(&self) -> Option<String> {
        self.auth_token.as_ref().map(|t| format!("Bearer {}", t))
    }

    // Health check
    pub async fn health_check(&self) -> Result<String, reqwest::Error> {
        let response = self.client.get(&format!("{}/health", self.base_url))
            .send()
            .await?;
        response.text().await
    }

    pub async fn ready_check(&self) -> Result<String, reqwest::Error> {
        let response = self.client.get(&format!("{}/ready", self.base_url))
            .send()
            .await?;
        response.text().await
    }

    // Auth endpoints
    pub async fn register(&self, email: &str, password: &str, company_name: Option<&str>) -> Result<reqwest::Response, reqwest::Error> {
        let mut body = serde_json::json!({
            "email": email,
            "password": password,
        });
        if let Some(name) = company_name {
            body["company_name"] = serde_json::json!(name);
        }

        self.client.post(&format!("{}/api/v1/auth/register", self.base_url))
            .json(&body)
            .send()
            .await
    }

    pub async fn login(&self, email: &str, password: &str) -> Result<reqwest::Response, reqwest::Error> {
        self.client.post(&format!("{}/api/v1/auth/login", self.base_url))
            .json(&serde_json::json!({
                "email": email,
                "password": password,
            }))
            .send()
            .await
    }

    pub async fn get_current_user(&self) -> Result<reqwest::Response, reqwest::Error> {
        let mut request = self.client.get(&format!("{}/api/v1/auth/me", self.base_url));
        if let Some(auth) = self.get_auth_header() {
            request = request.header("Authorization", auth);
        }
        request.send().await
    }

    pub async fn update_profile(&self, phone: &str, company_name: &str) -> Result<reqwest::Response, reqwest::Error> {
        let mut request = self.client.put(&format!("{}/api/v1/auth/profile", self.base_url))
            .json(&serde_json::json!({
                "phone": phone,
                "company_name": company_name,
            }));
        if let Some(auth) = self.get_auth_header() {
            request = request.header("Authorization", auth);
        }
        request.send().await
    }

    // Client endpoints
    pub async fn create_client(&self, name: &str, email: &str) -> Result<reqwest::Response, reqwest::Error> {
        let mut request = self.client.post(&format!("{}/api/v1/clients", self.base_url))
            .json(&serde_json::json!({
                "name": name,
                "email": email,
            }));
        if let Some(auth) = self.get_auth_header() {
            request = request.header("Authorization", auth);
        }
        request.send().await
    }

    pub async fn list_clients(&self) -> Result<reqwest::Response, reqwest::Error> {
        let mut request = self.client.get(&format!("{}/api/v1/clients", self.base_url));
        if let Some(auth) = self.get_auth_header() {
            request = request.header("Authorization", auth);
        }
        request.send().await
    }

    pub async fn get_client(&self, client_id: &str) -> Result<reqwest::Response, reqwest::Error> {
        let mut request = self.client.get(&format!("{}/api/v1/clients/{}", self.base_url, client_id));
        if let Some(auth) = self.get_auth_header() {
            request = request.header("Authorization", auth);
        }
        request.send().await
    }

    pub async fn update_client(&self, client_id: &str, name: &str, email: &str) -> Result<reqwest::Response, reqwest::Error> {
        let mut request = self.client.put(&format!("{}/api/v1/clients/{}", self.base_url, client_id))
            .json(&serde_json::json!({
                "name": name,
                "email": email,
            }));
        if let Some(auth) = self.get_auth_header() {
            request = request.header("Authorization", auth);
        }
        request.send().await
    }

    pub async fn delete_client(&self, client_id: &str) -> Result<reqwest::Response, reqwest::Error> {
        let mut request = self.client.delete(&format!("{}/api/v1/clients/{}", self.base_url, client_id));
        if let Some(auth) = self.get_auth_header() {
            request = request.header("Authorization", auth);
        }
        request.send().await
    }

    pub async fn get_client_invoices(&self, client_id: &str) -> Result<reqwest::Response, reqwest::Error> {
        let mut request = self.client.get(&format!("{}/api/v1/clients/{}/invoices", self.base_url, client_id));
        if let Some(auth) = self.get_auth_header() {
            request = request.header("Authorization", auth);
        }
        request.send().await
    }

    pub async fn get_client_stats(&self) -> Result<reqwest::Response, reqwest::Error> {
        let mut request = self.client.get(&format!("{}/api/v1/clients/stats", self.base_url));
        if let Some(auth) = self.get_auth_header() {
            request = request.header("Authorization", auth);
        }
        request.send().await
    }

    // Invoice endpoints
    pub async fn create_invoice(&self, client_id: &str, amount: f64) -> Result<reqwest::Response, reqwest::Error> {
        let mut request = self.client.post(&format!("{}/api/v1/invoices", self.base_url))
            .json(&serde_json::json!({
                "client_id": client_id,
                "items": [
                    {
                        "description": "Test Service",
                        "quantity": 1,
                        "unit_price": amount,
                        "tax_rate": 0.0
                    }
                ],
                "notes": "Test invoice",
                "terms": "Payment due in 30 days"
            }));
        if let Some(auth) = self.get_auth_header() {
            request = request.header("Authorization", auth);
        }
        request.send().await
    }

    pub async fn list_invoices(&self) -> Result<reqwest::Response, reqwest::Error> {
        let mut request = self.client.get(&format!("{}/api/v1/invoices", self.base_url));
        if let Some(auth) = self.get_auth_header() {
            request = request.header("Authorization", auth);
        }
        request.send().await
    }

    pub async fn get_invoice(&self, invoice_id: &str) -> Result<reqwest::Response, reqwest::Error> {
        let mut request = self.client.get(&format!("{}/api/v1/invoices/{}", self.base_url, invoice_id));
        if let Some(auth) = self.get_auth_header() {
            request = request.header("Authorization", auth);
        }
        request.send().await
    }

    pub async fn update_invoice(&self, invoice_id: &str, notes: &str) -> Result<reqwest::Response, reqwest::Error> {
        let mut request = self.client.put(&format!("{}/api/v1/invoices/{}", self.base_url, invoice_id))
            .json(&serde_json::json!({
                "notes": notes,
            }));
        if let Some(auth) = self.get_auth_header() {
            request = request.header("Authorization", auth);
        }
        request.send().await
    }

    pub async fn delete_invoice(&self, invoice_id: &str) -> Result<reqwest::Response, reqwest::Error> {
        let mut request = self.client.delete(&format!("{}/api/v1/invoices/{}", self.base_url, invoice_id));
        if let Some(auth) = self.get_auth_header() {
            request = request.header("Authorization", auth);
        }
        request.send().await
    }

    pub async fn send_invoice(&self, invoice_id: &str) -> Result<reqwest::Response, reqwest::Error> {
        let mut request = self.client.post(&format!("{}/api/v1/invoices/{}/send", self.base_url, invoice_id))
            .json(&serde_json::json!({}));
        if let Some(auth) = self.get_auth_header() {
            request = request.header("Authorization", auth);
        }
        request.send().await
    }

    pub async fn get_invoice_pdf(&self, invoice_id: &str) -> Result<reqwest::Response, reqwest::Error> {
        let mut request = self.client.get(&format!("{}/api/v1/invoices/{}/pdf", self.base_url, invoice_id));
        if let Some(auth) = self.get_auth_header() {
            request = request.header("Authorization", auth);
        }
        request.send().await
    }

    pub async fn send_reminder(&self, invoice_id: &str) -> Result<reqwest::Response, reqwest::Error> {
        let mut request = self.client.post(&format!("{}/api/v1/invoices/{}/reminder", self.base_url, invoice_id))
            .json(&serde_json::json!({}));
        if let Some(auth) = self.get_auth_header() {
            request = request.header("Authorization", auth);
        }
        request.send().await
    }

    pub async fn record_payment(&self, invoice_id: &str, amount: f64) -> Result<reqwest::Response, reqwest::Error> {
        let mut request = self.client.post(&format!("{}/api/v1/invoices/{}/payments", self.base_url, invoice_id))
            .json(&serde_json::json!({
                "amount": amount,
                "payment_method": "cash",
            }));
        if let Some(auth) = self.get_auth_header() {
            request = request.header("Authorization", auth);
        }
        request.send().await
    }

    // Payment endpoints
    pub async fn create_payment(&self, invoice_id: &str, amount: f64) -> Result<reqwest::Response, reqwest::Error> {
        let mut request = self.client.post(&format!("{}/api/v1/payments", self.base_url))
            .json(&serde_json::json!({
                "invoice_id": invoice_id,
                "amount": amount,
                "payment_method": "stripe",
            }));
        if let Some(auth) = self.get_auth_header() {
            request = request.header("Authorization", auth);
        }
        request.send().await
    }

    pub async fn list_payments(&self) -> Result<reqwest::Response, reqwest::Error> {
        let mut request = self.client.get(&format!("{}/api/v1/payments", self.base_url));
        if let Some(auth) = self.get_auth_header() {
            request = request.header("Authorization", auth);
        }
        request.send().await
    }

    pub async fn get_payment(&self, payment_id: &str) -> Result<reqwest::Response, reqwest::Error> {
        let mut request = self.client.get(&format!("{}/api/v1/payments/{}", self.base_url, payment_id));
        if let Some(auth) = self.get_auth_header() {
            request = request.header("Authorization", auth);
        }
        request.send().await
    }

    pub async fn refund_payment(&self, payment_id: &str, reason: &str) -> Result<reqwest::Response, reqwest::Error> {
        let mut request = self.client.post(&format!("{}/api/v1/payments/{}/refund", self.base_url, payment_id))
            .json(&serde_json::json!({
                "reason": reason,
            }));
        if let Some(auth) = self.get_auth_header() {
            request = request.header("Authorization", auth);
        }
        request.send().await
    }

    pub async fn get_payment_stats(&self) -> Result<reqwest::Response, reqwest::Error> {
        let mut request = self.client.get(&format!("{}/api/v1/payments/stats", self.base_url));
        if let Some(auth) = self.get_auth_header() {
            request = request.header("Authorization", auth);
        }
        request.send().await
    }

    pub async fn get_payment_methods(&self) -> Result<reqwest::Response, reqwest::Error> {
        let mut request = self.client.get(&format!("{}/api/v1/payments/methods", self.base_url));
        if let Some(auth) = self.get_auth_header() {
            request = request.header("Authorization", auth);
        }
        request.send().await
    }

    // Expense endpoints
    pub async fn create_expense(&self, amount: f64, category: &str, vendor: &str) -> Result<reqwest::Response, reqwest::Error> {
        let mut request = self.client.post(&format!("{}/api/v1/expenses", self.base_url))
            .json(&serde_json::json!({
                "amount": amount,
                "category": category,
                "vendor": vendor,
                "description": "Test expense",
                "date_incurred": "2025-01-01",
                "tax_deductible": true,
            }));
        if let Some(auth) = self.get_auth_header() {
            request = request.header("Authorization", auth);
        }
        request.send().await
    }

    pub async fn list_expenses(&self) -> Result<reqwest::Response, reqwest::Error> {
        let mut request = self.client.get(&format!("{}/api/v1/expenses", self.base_url));
        if let Some(auth) = self.get_auth_header() {
            request = request.header("Authorization", auth);
        }
        request.send().await
    }

    pub async fn get_expense(&self, expense_id: &str) -> Result<reqwest::Response, reqwest::Error> {
        let mut request = self.client.get(&format!("{}/api/v1/expenses/{}", self.base_url, expense_id));
        if let Some(auth) = self.get_auth_header() {
            request = request.header("Authorization", auth);
        }
        request.send().await
    }

    pub async fn update_expense(&self, expense_id: &str, description: &str) -> Result<reqwest::Response, reqwest::Error> {
        let mut request = self.client.put(&format!("{}/api/v1/expenses/{}", self.base_url, expense_id))
            .json(&serde_json::json!({
                "description": description,
            }));
        if let Some(auth) = self.get_auth_header() {
            request = request.header("Authorization", auth);
        }
        request.send().await
    }

    pub async fn delete_expense(&self, expense_id: &str) -> Result<reqwest::Response, reqwest::Error> {
        let mut request = self.client.delete(&format!("{}/api/v1/expenses/{}", self.base_url, expense_id));
        if let Some(auth) = self.get_auth_header() {
            request = request.header("Authorization", auth);
        }
        request.send().await
    }

    pub async fn get_expense_stats(&self) -> Result<reqwest::Response, reqwest::Error> {
        let mut request = self.client.get(&format!("{}/api/v1/expenses/stats", self.base_url));
        if let Some(auth) = self.get_auth_header() {
            request = request.header("Authorization", auth);
        }
        request.send().await
    }

    // Report endpoints
    pub async fn get_overview_stats(&self) -> Result<reqwest::Response, reqwest::Error> {
        let mut request = self.client.get(&format!("{}/api/v1/reports/overview", self.base_url));
        if let Some(auth) = self.get_auth_header() {
            request = request.header("Authorization", auth);
        }
        request.send().await
    }

    pub async fn get_income_report(&self, start_date: &str, end_date: &str) -> Result<reqwest::Response, reqwest::Error> {
        let mut request = self.client.get(&format!("{}/api/v1/reports/income?start_date={}&end_date={}", self.base_url, start_date, end_date));
        if let Some(auth) = self.get_auth_header() {
            request = request.header("Authorization", auth);
        }
        request.send().await
    }

    pub async fn get_expenses_report(&self, start_date: &str, end_date: &str) -> Result<reqwest::Response, reqwest::Error> {
        let mut request = self.client.get(&format!("{}/api/v1/reports/expenses?start_date={}&end_date={}", self.base_url, start_date, end_date));
        if let Some(auth) = self.get_auth_header() {
            request = request.header("Authorization", auth);
        }
        request.send().await
    }

    pub async fn get_tax_report(&self, start_date: &str, end_date: &str) -> Result<reqwest::Response, reqwest::Error> {
        let mut request = self.client.get(&format!("{}/api/v1/reports/tax?start_date={}&end_date={}", self.base_url, start_date, end_date));
        if let Some(auth) = self.get_auth_header() {
            request = request.header("Authorization", auth);
        }
        request.send().await
    }

    pub async fn get_aging_report(&self) -> Result<reqwest::Response, reqwest::Error> {
        let mut request = self.client.get(&format!("{}/api/v1/reports/aging", self.base_url));
        if let Some(auth) = self.get_auth_header() {
            request = request.header("Authorization", auth);
        }
        request.send().await
    }

    pub async fn export_report(&self, report_type: &str, format: &str, start_date: &str, end_date: &str) -> Result<reqwest::Response, reqwest::Error> {
        let mut request = self.client.post(&format!("{}/api/v1/reports/export", self.base_url))
            .json(&serde_json::json!({
                "report_type": report_type,
                "format": format,
                "date_range": {
                    "start_date": start_date,
                    "end_date": end_date,
                },
            }));
        if let Some(auth) = self.get_auth_header() {
            request = request.header("Authorization", auth);
        }
        request.send().await
    }

    // Settings endpoints
    pub async fn get_business_settings(&self) -> Result<reqwest::Response, reqwest::Error> {
        let mut request = self.client.get(&format!("{}/api/v1/settings/business", self.base_url));
        if let Some(auth) = self.get_auth_header() {
            request = request.header("Authorization", auth);
        }
        request.send().await
    }

    pub async fn update_business_settings(&self, company_name: &str, phone: &str) -> Result<reqwest::Response, reqwest::Error> {
        let mut request = self.client.put(&format!("{}/api/v1/settings/business", self.base_url))
            .json(&serde_json::json!({
                "company_name": company_name,
                "phone": phone,
            }));
        if let Some(auth) = self.get_auth_header() {
            request = request.header("Authorization", auth);
        }
        request.send().await
    }

    pub async fn get_tax_settings(&self) -> Result<reqwest::Response, reqwest::Error> {
        let mut request = self.client.get(&format!("{}/api/v1/settings/tax", self.base_url));
        if let Some(auth) = self.get_auth_header() {
            request = request.header("Authorization", auth);
        }
        request.send().await
    }

    pub async fn update_tax_settings(&self, state_code: &str, tax_rate: f64) -> Result<reqwest::Response, reqwest::Error> {
        let mut request = self.client.put(&format!("{}/api/v1/settings/tax", self.base_url))
            .json(&serde_json::json!({
                "state_code": state_code,
                "tax_rate": tax_rate,
                "tax_exempt": false,
            }));
        if let Some(auth) = self.get_auth_header() {
            request = request.header("Authorization", auth);
        }
        request.send().await
    }

    pub async fn get_notification_settings(&self) -> Result<reqwest::Response, reqwest::Error> {
        let mut request = self.client.get(&format!("{}/api/v1/settings/notifications", self.base_url));
        if let Some(auth) = self.get_auth_header() {
            request = request.header("Authorization", auth);
        }
        request.send().await
    }

    pub async fn update_notification_settings(&self) -> Result<reqwest::Response, reqwest::Error> {
        let mut request = self.client.put(&format!("{}/api/v1/settings/notifications", self.base_url))
            .json(&serde_json::json!({
                "email_payment_received": true,
                "email_invoice_paid": true,
                "email_payment_reminder": true,
                "push_payment_received": true,
                "push_overdue": true,
            }));
        if let Some(auth) = self.get_auth_header() {
            request = request.header("Authorization", auth);
        }
        request.send().await
    }

    pub async fn get_invoice_settings(&self) -> Result<reqwest::Response, reqwest::Error> {
        let mut request = self.client.get(&format!("{}/api/v1/settings/invoice", self.base_url));
        if let Some(auth) = self.get_auth_header() {
            request = request.header("Authorization", auth);
        }
        request.send().await
    }

    pub async fn update_invoice_settings(&self, template: &str, terms: &str, notes: &str) -> Result<reqwest::Response, reqwest::Error> {
        let mut request = self.client.put(&format!("{}/api/v1/settings/invoice", self.base_url))
            .json(&serde_json::json!({
                "template": template,
                "terms": terms,
                "notes": notes,
            }));
        if let Some(auth) = self.get_auth_header() {
            request = request.header("Authorization", auth);
        }
        request.send().await
    }
}
