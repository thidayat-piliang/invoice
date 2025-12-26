use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NotificationError {
    #[error("Notification service error: {0}")]
    ServiceError(String),
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

pub struct NotificationService;

impl NotificationService {
    pub fn new() -> Self {
        Self
    }

    pub fn send_push_notification(
        &self,
        _token: &str,
        _title: &str,
        _body: &str,
        _data: Option<serde_json::Value>,
    ) -> Result<(), NotificationError> {
        // Firebase Cloud Messaging integration point
        // Requires FCM credentials and configuration
        // Returns success for now (can be implemented when FCM is configured)
        Ok(())
    }

    pub fn send_in_app_notification(
        &self,
        user_id: &str,
        title: &str,
        body: &str,
        notification_type: NotificationType,
        data: Option<serde_json::Value>,
    ) -> Result<(), NotificationError> {
        // Store notification in database for in-app display
        // This would typically be stored in a notifications table
        Ok(())
    }

    pub fn send_payment_received_notification(
        &self,
        user_id: &str,
        invoice_number: &str,
        amount: f64,
        push: bool,
        email: bool,
    ) -> Result<(), NotificationError> {
        let title = "Payment Received!";
        let body = format!("Payment of ${:.2} received for invoice #{}", amount, invoice_number);

        if push {
            // Send push notification
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
        }

        Ok(())
    }

    pub fn send_overdue_notification(
        &self,
        user_id: &str,
        invoice_number: &str,
        days_overdue: i64,
        amount_due: f64,
    ) -> Result<(), NotificationError> {
        let title = "Invoice Overdue";
        let body = format!(
            "Invoice #{} is {} days overdue. Amount due: ${:.2}",
            invoice_number, days_overdue, amount_due
        );

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
        )
    }

    pub fn send_invoice_created_notification(
        &self,
        user_id: &str,
        invoice_number: &str,
        client_name: &str,
        amount: f64,
    ) -> Result<(), NotificationError> {
        let title = "Invoice Created";
        let body = format!(
            "Invoice #{} created for {} - ${:.2}",
            invoice_number, client_name, amount
        );

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
        )
    }
}
