use crate::domain::models::invoice::InvoiceDetailResponse;
use crate::domain::models::user::User;
use anyhow::Result;
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppConfig {
    pub api_key: Option<String>,
    pub api_url: Option<String>,
    pub sender_name: Option<String>,
    pub enabled: bool,
}

impl Default for WhatsAppConfig {
    fn default() -> Self {
        Self {
            api_key: env::var("WHATSAPP_API_KEY").ok(),
            api_url: env::var("WHATSAPP_API_URL").ok(),
            sender_name: env::var("WHATSAPP_SENDER_NAME").ok(),
            enabled: env::var("WHATSAPP_ENABLED").unwrap_or_else(|_| "false".to_string()) == "true",
        }
    }
}

#[derive(Debug, Clone)]
pub struct WhatsAppService {
    http_client: HttpClient,
    config: WhatsAppConfig,
}

#[derive(Debug, Serialize)]
struct WhatsAppMessage {
    to: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    preview_url: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct WhatsAppResponse {
    pub success: bool,
    pub message_id: Option<String>,
    pub error: Option<String>,
}

impl WhatsAppService {
    pub fn new(config: WhatsAppConfig) -> Self {
        Self {
            http_client: HttpClient::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| HttpClient::new()),
            config,
        }
    }

    pub fn from_env() -> Self {
        Self::new(WhatsAppConfig::default())
    }

    /// Check if WhatsApp is configured and enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
            && self.config.api_key.is_some()
            && self.config.api_url.is_some()
    }

    /// Send invoice via WhatsApp
    pub async fn send_invoice(
        &self,
        phone: &str,
        invoice: &InvoiceDetailResponse,
        user: &User,
        payment_link: Option<String>,
    ) -> Result<WhatsAppResponse> {
        if !self.is_enabled() {
            return Ok(WhatsAppResponse {
                success: false,
                message_id: None,
                error: Some("WhatsApp not configured".to_string()),
            });
        }

        let message = self.create_invoice_message(invoice, user, payment_link);

        let payload = WhatsAppMessage {
            to: self.normalize_phone(phone),
            message,
            preview_url: Some(true),
        };

        let response = self
            .http_client
            .post(self.config.api_url.as_ref().unwrap())
            .header("Authorization", format!("Bearer {}", self.config.api_key.as_ref().unwrap()))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            let resp: serde_json::Value = response.json().await?;
            Ok(WhatsAppResponse {
                success: true,
                message_id: resp.get("message_id").and_then(|v| v.as_str()).map(String::from),
                error: None,
            })
        } else {
            let error_text = response.text().await?;
            Ok(WhatsAppResponse {
                success: false,
                message_id: None,
                error: Some(error_text),
            })
        }
    }

    /// Send guest checkout link via WhatsApp
    pub async fn send_guest_payment_link(
        &self,
        phone: &str,
        invoice: &InvoiceDetailResponse,
        payment_link: &str,
        recent_payments_count: usize,
    ) -> Result<WhatsAppResponse> {
        if !self.is_enabled() {
            return Ok(WhatsAppResponse {
                success: false,
                message_id: None,
                error: Some("WhatsApp not configured".to_string()),
            });
        }

        let message = self.create_guest_payment_message(invoice, payment_link, recent_payments_count);

        let payload = WhatsAppMessage {
            to: self.normalize_phone(phone),
            message,
            preview_url: Some(true),
        };

        let response = self
            .http_client
            .post(self.config.api_url.as_ref().unwrap())
            .header("Authorization", format!("Bearer {}", self.config.api_key.as_ref().unwrap()))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            let resp: serde_json::Value = response.json().await?;
            Ok(WhatsAppResponse {
                success: true,
                message_id: resp.get("message_id").and_then(|v| v.as_str()).map(String::from),
                error: None,
            })
        } else {
            let error_text = response.text().await?;
            Ok(WhatsAppResponse {
                success: false,
                message_id: None,
                error: Some(error_text),
            })
        }
    }

    /// Send reminder if invoice not viewed after X days
    pub async fn send_unviewed_reminder(
        &self,
        phone: &str,
        invoice: &InvoiceDetailResponse,
        days_unviewed: i64,
    ) -> Result<WhatsAppResponse> {
        if !self.is_enabled() {
            return Ok(WhatsAppResponse {
                success: false,
                message_id: None,
                error: Some("WhatsApp not configured".to_string()),
            });
        }

        let message = format!(
            "📝 *Invoice Reminder*\n\n\
            Hi! Invoice *{}* has been waiting for you for {} days.\n\n\
            Amount: ${:.2}\n\
            Due Date: {}\n\n\
            Please review and make payment at your earliest convenience.\n\n\
            View invoice: https://yourapp.com/invoices/{}\n\n\
            Thank you!",
            invoice.invoice_number,
            days_unviewed,
            invoice.total_amount,
            invoice.due_date,
            invoice.id
        );

        let payload = WhatsAppMessage {
            to: self.normalize_phone(phone),
            message,
            preview_url: Some(true),
        };

        let response = self
            .http_client
            .post(self.config.api_url.as_ref().unwrap())
            .header("Authorization", format!("Bearer {}", self.config.api_key.as_ref().unwrap()))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            let resp: serde_json::Value = response.json().await?;
            Ok(WhatsAppResponse {
                success: true,
                message_id: resp.get("message_id").and_then(|v| v.as_str()).map(String::from),
                error: None,
            })
        } else {
            let error_text = response.text().await?;
            Ok(WhatsAppResponse {
                success: false,
                message_id: None,
                error: Some(error_text),
            })
        }
    }

    /// Create invoice message for registered buyer
    fn create_invoice_message(
        &self,
        invoice: &InvoiceDetailResponse,
        user: &User,
        payment_link: Option<String>,
    ) -> String {
        let sender_name = self.config.sender_name.as_deref().unwrap_or("FlashBill");

        let mut message = format!(
            "📝 *New Invoice from {}*\n\n\
            Invoice Number: {}\n\
            Amount: ${:.2}\n\
            Due Date: {}\n\n",
            sender_name,
            invoice.invoice_number,
            invoice.total_amount,
            invoice.due_date
        );

        if let Some(client_name) = user.company_name.as_ref() {
            message.push_str(&format!("From: {}\n\n", client_name));
        }

        if let Some(link) = payment_link {
            message.push_str(&format!("💰 *Pay Now:* {}\n\n", link));
        }

        message.push_str(&format!(
            "📱 You can also view this invoice in the FlashBill app.\n\
            Download: https://yourapp.com/download\n\n\
            Thank you for your business! 🙏"
        ));

        message
    }

    /// Create guest payment message
    fn create_guest_payment_message(
        &self,
        invoice: &InvoiceDetailResponse,
        payment_link: &str,
        recent_payments_count: usize,
    ) -> String {
        let mut message = format!(
            "📝 *Invoice Payment Request*\n\n\
            Invoice Number: {}\n\
            Amount: ${:.2}\n\
            Due Date: {}\n\n\
            💰 *Pay Now:* {}\n\n",
            invoice.invoice_number,
            invoice.total_amount,
            invoice.due_date,
            payment_link
        );

        // Add payment history if available
        if recent_payments_count > 0 {
            message.push_str(&format!(
                "📊 Your recent payment history: {} payments\n\n",
                recent_payments_count
            ));
        }

        // Add app download promotion
        message.push_str(
            "📱 *Want to track all your payments?*\n\
            Download FlashBill app to:\n\
            • View all payment history\n\
            • Get payment reminders\n\
            • Manage invoices easily\n\n\
            Download: https://yourapp.com/download\n\n\
            Thank you! 🙏"
        );

        message
    }

    /// Normalize phone number (remove +, spaces, etc.)
    fn normalize_phone(&self, phone: &str) -> String {
        phone
            .chars()
            .filter(|c| c.is_ascii_digit())
            .collect()
    }

    /// Send payment confirmation
    pub async fn send_payment_confirmation(
        &self,
        phone: &str,
        invoice: &InvoiceDetailResponse,
    ) -> Result<WhatsAppResponse> {
        if !self.is_enabled() {
            return Ok(WhatsAppResponse {
                success: false,
                message_id: None,
                error: Some("WhatsApp not configured".to_string()),
            });
        }

        let message = format!(
            "✅ *Payment Received*\n\n\
            Invoice Number: {}\n\
            Amount Paid: ${:.2}\n\
            Status: PAID\n\n\
            Thank you for your payment! 🙏\n\n\
            View receipt: https://yourapp.com/invoices/{}",
            invoice.invoice_number,
            invoice.total_amount,
            invoice.id
        );

        let payload = WhatsAppMessage {
            to: self.normalize_phone(phone),
            message,
            preview_url: Some(true),
        };

        let response = self
            .http_client
            .post(self.config.api_url.as_ref().unwrap())
            .header("Authorization", format!("Bearer {}", self.config.api_key.as_ref().unwrap()))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            let resp: serde_json::Value = response.json().await?;
            Ok(WhatsAppResponse {
                success: true,
                message_id: resp.get("message_id").and_then(|v| v.as_str()).map(String::from),
                error: None,
            })
        } else {
            let error_text = response.text().await?;
            Ok(WhatsAppResponse {
                success: false,
                message_id: None,
                error: Some(error_text),
            })
        }
    }
}
