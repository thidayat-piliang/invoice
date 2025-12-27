use crate::domain::models::invoice::InvoiceDetailResponse;
use crate::domain::models::user::User;
use crate::domain::services::email_service::EmailService;
use crate::domain::services::whatsapp_service::WhatsAppService;
use anyhow::Result;
use chrono::Utc;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct EnhancedNotificationService {
    email_service: Arc<EmailService>,
    whatsapp_service: Arc<WhatsAppService>,
}

#[derive(Debug, Default, Clone)]
pub struct NotificationResult {
    pub email_sent: bool,
    pub email_message_id: Option<String>,
    pub email_error: Option<String>,
    pub whatsapp_sent: bool,
    pub whatsapp_message_id: Option<String>,
    pub whatsapp_error: Option<String>,
}

impl NotificationResult {
    pub fn is_success(&self) -> bool {
        (self.email_sent || self.whatsapp_sent) && self.email_error.is_none() && self.whatsapp_error.is_none()
    }
}

impl EnhancedNotificationService {
    pub fn new(
        email_service: Arc<EmailService>,
        whatsapp_service: Arc<WhatsAppService>,
    ) -> Self {
        Self {
            email_service,
            whatsapp_service,
        }
    }

    /// Send invoice notification to buyer (registered)
    pub async fn send_invoice_notification(
        &self,
        user: &User,
        invoice: &InvoiceDetailResponse,
        recipient_email: Option<String>,
        recipient_phone: Option<String>,
        payment_link: Option<String>,
    ) -> Result<NotificationResult> {
        let mut result = NotificationResult::default();

        // Send email if available
        if let Some(email) = recipient_email.clone() {
            let user_name = user.company_name.as_deref().unwrap_or("Unknown");
            let pdf_url = payment_link.clone().unwrap_or_else(|| format!("https://yourapp.com/invoices/{}", invoice.id));

            match self
                .email_service
                .send_invoice(&email, user_name, &invoice.invoice_number, &pdf_url, invoice.total_amount, &invoice.due_date.to_string())
            {
                Ok(_) => {
                    result.email_sent = true;
                    result.email_message_id = Some(uuid::Uuid::new_v4().to_string());
                }
                Err(e) => {
                    result.email_error = Some(e.to_string());
                }
            }
        }

        // Send WhatsApp if available and enabled
        if let Some(phone) = recipient_phone.clone() {
            match self
                .whatsapp_service
                .send_invoice(&phone, invoice, user, payment_link)
                .await
            {
                Ok(resp) => {
                    if resp.success {
                        result.whatsapp_sent = true;
                        result.whatsapp_message_id = resp.message_id;
                    } else {
                        result.whatsapp_error = resp.error;
                    }
                }
                Err(e) => {
                    result.whatsapp_error = Some(e.to_string());
                }
            }
        }

        Ok(result)
    }

    /// Send guest checkout notification
    pub async fn send_guest_notification(
        &self,
        invoice: &InvoiceDetailResponse,
        recipient_phone: Option<String>,
        payment_link: String,
        recent_payments_count: usize,
    ) -> Result<NotificationResult> {
        let mut result = NotificationResult::default();

        // Send WhatsApp for guest
        if let Some(phone) = recipient_phone {
            match self
                .whatsapp_service
                .send_guest_payment_link(&phone, invoice, &payment_link, recent_payments_count)
                .await
            {
                Ok(resp) => {
                    if resp.success {
                        result.whatsapp_sent = true;
                        result.whatsapp_message_id = resp.message_id;
                    } else {
                        result.whatsapp_error = resp.error;
                    }
                }
                Err(e) => {
                    result.whatsapp_error = Some(e.to_string());
                }
            }
        }

        Ok(result)
    }

    /// Send payment confirmation
    pub async fn send_payment_confirmation(
        &self,
        invoice: &InvoiceDetailResponse,
        recipient_email: Option<String>,
        recipient_phone: Option<String>,
    ) -> Result<NotificationResult> {
        let mut result = NotificationResult::default();

        // Email confirmation
        if let Some(email) = recipient_email {
            let client_name = invoice.client_name.clone();
            let payment_method = "PayPal"; // Default, can be parameterized

            match self
                .email_service
                .send_payment_confirmation(&email, &client_name, &invoice.invoice_number, invoice.total_amount, payment_method)
            {
                Ok(_) => {
                    result.email_sent = true;
                    result.email_message_id = Some(uuid::Uuid::new_v4().to_string());
                }
                Err(e) => {
                    result.email_error = Some(e.to_string());
                }
            }
        }

        // WhatsApp confirmation
        if let Some(phone) = recipient_phone {
            match self
                .whatsapp_service
                .send_payment_confirmation(&phone, invoice)
                .await
            {
                Ok(resp) => {
                    if resp.success {
                        result.whatsapp_sent = true;
                        result.whatsapp_message_id = resp.message_id;
                    } else {
                        result.whatsapp_error = resp.error;
                    }
                }
                Err(e) => {
                    result.whatsapp_error = Some(e.to_string());
                }
            }
        }

        Ok(result)
    }

    /// Send unviewed reminder (if invoice not viewed after X days)
    pub async fn send_unviewed_reminder(
        &self,
        invoice: &InvoiceDetailResponse,
        recipient_phone: Option<String>,
        days_unviewed: i64,
    ) -> Result<NotificationResult> {
        let mut result = NotificationResult::default();

        if let Some(phone) = recipient_phone {
            match self
                .whatsapp_service
                .send_unviewed_reminder(&phone, invoice, days_unviewed)
                .await
            {
                Ok(resp) => {
                    if resp.success {
                        result.whatsapp_sent = true;
                        result.whatsapp_message_id = resp.message_id;
                    } else {
                        result.whatsapp_error = resp.error;
                    }
                }
                Err(e) => {
                    result.whatsapp_error = Some(e.to_string());
                }
            }
        }

        Ok(result)
    }

    /// Check if notification should be sent based on read status
    pub fn should_send_notification(
        &self,
        invoice: &InvoiceDetailResponse,
        days_unviewed: i64,
    ) -> bool {
        // Only send if invoice not viewed and has been sent for X days
        invoice.viewed_at.is_none()
            && invoice.sent_at.is_some()
            && (Utc::now() - invoice.sent_at.unwrap()).num_days() >= days_unviewed
    }
}
