#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NotificationError {
    #[error("Notification service error: {0}")]
    ServiceError(String),
    #[error("FCM error: {0}")]
    FcmError(String),
    #[error("HTTP error: {0}")]
    HttpError(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationType {
    PaymentReceived,
    InvoicePaid,
    PaymentReminder,
    OverdueNotice,
    InvoiceCreated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub user_id: String,
    pub title: String,
    pub body: String,
    pub notification_type: NotificationType,
    pub data: Option<serde_json::Value>,
}

/// FCM (Firebase Cloud Messaging) request payload
#[derive(Debug, Serialize)]
struct FcmRequest {
    to: String,
    notification: FcmNotification,
    data: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct FcmNotification {
    title: String,
    body: String,
}

/// FCM response
#[derive(Debug, Deserialize)]
struct FcmResponse {
    name: Option<String>,
    error: Option<FcmError>,
}

#[derive(Debug, Deserialize)]
struct FcmError {
    code: i32,
    message: String,
}

pub struct NotificationService {
    fcm_server_key: Option<String>,
    http_client: reqwest::Client,
}

impl NotificationService {
    pub fn new() -> Result<Self, NotificationError> {
        let fcm_server_key = std::env::var("FCM_SERVER_KEY").ok();

        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| NotificationError::HttpError(e.to_string()))?;

        Ok(Self {
            fcm_server_key,
            http_client,
        })
    }

    /// Send push notification via FCM (Firebase Cloud Messaging)
    pub async fn send_push_notification(
        &self,
        token: &str,
        title: &str,
        body: &str,
        data: Option<serde_json::Value>,
    ) -> Result<(), NotificationError> {
        let server_key = self.fcm_server_key.as_ref()
            .ok_or_else(|| NotificationError::ConfigError("FCM server key not configured".to_string()))?;

        let payload = FcmRequest {
            to: token.to_string(),
            notification: FcmNotification {
                title: title.to_string(),
                body: body.to_string(),
            },
            data,
        };

        // Send request to FCM
        let response = self.http_client
            .post("https://fcm.googleapis.com/fcm/send")
            .header("Authorization", format!("key={}", server_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| NotificationError::HttpError(e.to_string()))?;

        let status = response.status();
        if status.is_success() {
            let fcm_response: FcmResponse = response.json()
                .await
                .map_err(|e| NotificationError::FcmError(e.to_string()))?;

            if let Some(error) = fcm_response.error {
                return Err(NotificationError::FcmError(format!(
                    "FCM error {}: {}", error.code, error.message
                )));
            }

            Ok(())
        } else {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(NotificationError::FcmError(format!("HTTP {}: {}", status, error_text)))
        }
    }

    /// Send in-app notification (stored in database)
    pub fn send_in_app_notification(
        &self,
        user_id: &str,
        title: &str,
        body: &str,
        notification_type: NotificationType,
        _data: Option<serde_json::Value>,
    ) -> Result<(), NotificationError> {
        // Store notification in database for in-app display
        // This would typically be stored in a notifications table
        // For now, we just log it
        tracing::info!(
            "In-app notification for user {}: {} - {} (type: {:?})",
            user_id,
            title,
            body,
            notification_type
        );
        Ok(())
    }

    /// Send payment received notification (push + in-app)
    pub async fn send_payment_received_notification(
        &self,
        user_id: &str,
        fcm_token: Option<&str>,
        invoice_number: &str,
        amount: f64,
    ) -> Result<(), NotificationError> {
        let title = "Payment Received!";
        let body = format!("Payment of ${:.2} received for invoice #{}", amount, invoice_number);

        // Send in-app notification
        self.send_in_app_notification(
            user_id,
            title,
            &body,
            NotificationType::PaymentReceived,
            Some(serde_json::json!({
                "invoice_number": invoice_number,
                "amount": amount,
                "type": "payment_received"
            })),
        )?;

        // Send push notification if token is provided
        if let Some(token) = fcm_token {
            if !token.is_empty() {
                let _ = self.send_push_notification(
                    token,
                    title,
                    &body,
                    Some(serde_json::json!({
                        "invoice_number": invoice_number,
                        "amount": amount,
                        "type": "payment_received"
                    })),
                ).await;
            }
        }

        Ok(())
    }

    /// Send overdue notification
    pub async fn send_overdue_notification(
        &self,
        user_id: &str,
        fcm_token: Option<&str>,
        invoice_number: &str,
        days_overdue: i64,
        amount_due: f64,
    ) -> Result<(), NotificationError> {
        let title = "Invoice Overdue";
        let body = format!(
            "Invoice #{} is {} days overdue. Amount due: ${:.2}",
            invoice_number, days_overdue, amount_due
        );

        // Send in-app notification
        self.send_in_app_notification(
            user_id,
            title,
            &body,
            NotificationType::OverdueNotice,
            Some(serde_json::json!({
                "invoice_number": invoice_number,
                "days_overdue": days_overdue,
                "amount_due": amount_due,
                "type": "overdue"
            })),
        )?;

        // Send push notification if token is provided
        if let Some(token) = fcm_token {
            if !token.is_empty() {
                let _ = self.send_push_notification(
                    token,
                    title,
                    &body,
                    Some(serde_json::json!({
                        "invoice_number": invoice_number,
                        "days_overdue": days_overdue,
                        "amount_due": amount_due,
                        "type": "overdue"
                    })),
                ).await;
            }
        }

        Ok(())
    }

    /// Send invoice created notification
    pub async fn send_invoice_created_notification(
        &self,
        user_id: &str,
        fcm_token: Option<&str>,
        invoice_number: &str,
        client_name: &str,
        amount: f64,
    ) -> Result<(), NotificationError> {
        let title = "Invoice Created";
        let body = format!(
            "Invoice #{} created for {} - ${:.2}",
            invoice_number, client_name, amount
        );

        // Send in-app notification
        self.send_in_app_notification(
            user_id,
            title,
            &body,
            NotificationType::InvoiceCreated,
            Some(serde_json::json!({
                "invoice_number": invoice_number,
                "client_name": client_name,
                "amount": amount,
                "type": "invoice_created"
            })),
        )?;

        // Send push notification if token is provided
        if let Some(token) = fcm_token {
            if !token.is_empty() {
                let _ = self.send_push_notification(
                    token,
                    title,
                    &body,
                    Some(serde_json::json!({
                        "invoice_number": invoice_number,
                        "client_name": client_name,
                        "amount": amount,
                        "type": "invoice_created"
                    })),
                ).await;
            }
        }

        Ok(())
    }

    /// Check if FCM is configured
    pub fn is_fcm_configured(&self) -> bool {
        self.fcm_server_key.is_some()
    }
}
