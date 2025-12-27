#![allow(dead_code)]

use crate::domain::models::*;
use crate::domain::services::{PdfService, InvoiceItemPdf, EmailService, EnhancedNotificationService, WhatsAppService};
use crate::infrastructure::repositories::{InvoiceRepository, ClientRepository, UserRepository};
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum InvoiceError {
    #[error("Invoice not found")]
    NotFound,

    #[error("Client not found")]
    ClientNotFound,

    #[error("Invalid invoice status: {0}")]
    InvalidStatus(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("PDF generation error: {0}")]
    PdfGenerationError(String),

    #[error("Email error: {0}")]
    EmailError(String),

    #[error("WhatsApp error: {0}")]
    WhatsAppError(String),

    #[error("Notification error: {0}")]
    NotificationError(String),
}

impl From<sqlx::Error> for InvoiceError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => InvoiceError::NotFound,
            _ => InvoiceError::DatabaseError(err.to_string()),
        }
    }
}

impl From<crate::domain::services::PdfError> for InvoiceError {
    fn from(err: crate::domain::services::PdfError) -> Self {
        InvoiceError::PdfGenerationError(err.to_string())
    }
}

impl From<crate::domain::services::EmailError> for InvoiceError {
    fn from(err: crate::domain::services::EmailError) -> Self {
        InvoiceError::EmailError(err.to_string())
    }
}

impl From<anyhow::Error> for InvoiceError {
    fn from(err: anyhow::Error) -> Self {
        InvoiceError::NotificationError(err.to_string())
    }
}

pub struct InvoiceService {
    invoice_repo: InvoiceRepository,
    client_repo: ClientRepository,
    user_repo: UserRepository,
    pdf_service: PdfService,
    email_service: Arc<EmailService>,
    notification_service: Arc<EnhancedNotificationService>,
    whatsapp_service: Arc<WhatsAppService>,
}

impl InvoiceService {
    pub fn new(
        invoice_repo: InvoiceRepository,
        client_repo: ClientRepository,
        user_repo: UserRepository,
        pdf_service: PdfService,
        email_service: Arc<EmailService>,
        notification_service: Arc<EnhancedNotificationService>,
        whatsapp_service: Arc<WhatsAppService>,
    ) -> Self {
        Self {
            invoice_repo,
            client_repo,
            user_repo,
            pdf_service,
            email_service,
            notification_service,
            whatsapp_service,
        }
    }

    pub async fn create_invoice(
        &self,
        user_id: Uuid,
        create: CreateInvoice,
    ) -> Result<InvoiceDetailResponse, InvoiceError> {
        // Validate client exists
        let client_opt = self.client_repo.find_by_id(user_id, create.client_id).await?;
        if client_opt.is_none() {
            return Err(InvoiceError::ClientNotFound);
        }

        // Create invoice via repository
        let invoice = self.invoice_repo.create(user_id, create).await?;

        // Get full details with client info
        let detail = self.invoice_repo.get_by_id(user_id, invoice.id).await?;

        Ok(detail)
    }

    pub async fn get_invoice(
        &self,
        user_id: Uuid,
        invoice_id: Uuid,
    ) -> Result<InvoiceDetailResponse, InvoiceError> {
        Ok(self.invoice_repo.get_by_id(user_id, invoice_id).await?)
    }

    pub async fn list_invoices(
        &self,
        user_id: Uuid,
        filter: InvoiceListFilter,
    ) -> Result<Vec<InvoiceResponse>, InvoiceError> {
        Ok(self.invoice_repo.list(user_id, filter).await?)
    }

    pub async fn update_invoice(
        &self,
        user_id: Uuid,
        invoice_id: Uuid,
        update: UpdateInvoice,
    ) -> Result<InvoiceDetailResponse, InvoiceError> {
        // Validate client exists if being updated
        if let Some(client_id) = update.client_id {
            let client_opt = self.client_repo.find_by_id(user_id, client_id).await?;
            if client_opt.is_none() {
                return Err(InvoiceError::ClientNotFound);
            }
        }

        // Update via repository
        let invoice = self.invoice_repo.update(user_id, invoice_id, update).await?;

        // Get full details
        let detail = self.invoice_repo.get_by_id(user_id, invoice.id).await?;

        Ok(detail)
    }

    pub async fn delete_invoice(
        &self,
        user_id: Uuid,
        invoice_id: Uuid,
    ) -> Result<(), InvoiceError> {
        Ok(self.invoice_repo.delete(user_id, invoice_id).await?)
    }

    pub async fn record_payment(
        &self,
        user_id: Uuid,
        invoice_id: Uuid,
        payment: CreatePayment,
    ) -> Result<InvoiceDetailResponse, InvoiceError> {
        // Record payment via repository
        let invoice = self.invoice_repo.record_payment(user_id, invoice_id, payment).await?;

        // Get full details
        let detail = self.invoice_repo.get_by_id(user_id, invoice.id).await?;

        Ok(detail)
    }

    pub async fn send_invoice(
        &self,
        user_id: Uuid,
        invoice_id: Uuid,
        email: Option<String>,
    ) -> Result<(), InvoiceError> {
        // Validate invoice exists and get details
        let detail = self.invoice_repo.get_by_id(user_id, invoice_id).await?;

        // Get user (company) info
        let user = self.user_repo.find_by_id(user_id)
            .await?
            .ok_or(InvoiceError::Validation("User not found".to_string()))?;

        // Get client info
        let client = self.client_repo.find_by_id(user_id, detail.client_id)
            .await?
            .ok_or(InvoiceError::ClientNotFound)?;

        // Generate PDF
        let items: Vec<InvoiceItemPdf> = detail.items.iter().map(|item| InvoiceItemPdf {
            description: item.description.clone(),
            quantity: item.quantity,
            unit_price: item.unit_price,
            total: item.total,
        }).collect();

        let company_address = user.business_address.as_ref().map(|addr| {
            format!("{}, {}, {} {}", addr.street, addr.city, addr.state, addr.zip_code)
        });

        let client_address = client.billing_address.as_ref().map(|addr| {
            addr.to_string()
        });

        let pdf_bytes = self.pdf_service.generate_invoice_pdf(
            &detail.invoice_number,
            user.company_name.as_deref(),
            company_address.as_deref(),
            &client.name,
            client.email.as_deref(),
            client_address.as_deref(),
            &detail.issue_date.to_string(),
            &detail.due_date.to_string(),
            &items,
            detail.subtotal,
            detail.tax_amount,
            detail.discount_amount,
            detail.total_amount,
            detail.notes.as_deref(),
            detail.terms.as_deref(),
            detail.tax_label.as_deref(),
        )?;

        // Send email with PDF attachment
        let email_sent = if let Some(email) = email.clone().or_else(|| detail.client_email.clone()) {
            self.email_service.send_invoice_with_attachment(
                &email,
                &client.name,
                &detail.invoice_number,
                pdf_bytes.clone(),
                detail.total_amount,
                &detail.due_date.to_string(),
            ).is_ok()
        } else {
            false
        };

        // Send WhatsApp notification if phone is available
        let whatsapp_sent = if let Some(phone) = client.phone.clone() {
            let payment_link = detail.guest_payment_token.clone()
                .map(|token| format!("https://yourapp.com/guest/pay/{}", token));
            let result = self.whatsapp_service.send_invoice(
                &phone,
                &detail,
                &user,
                payment_link,
            ).await;
            result.is_ok()
        } else {
            false
        };

        // Update notification tracking in repository
        self.invoice_repo.send_invoice(user_id, invoice_id, email.clone()).await?;

        // Update notification sent timestamps
        if email_sent {
            let _ = self.invoice_repo.update_notification_sent(invoice_id, false).await;
        }
        if whatsapp_sent {
            let _ = self.invoice_repo.update_notification_sent(invoice_id, true).await;
        }

        Ok(())
    }

    /// Send invoice via WhatsApp only
    pub async fn send_invoice_whatsapp(
        &self,
        user_id: Uuid,
        invoice_id: Uuid,
    ) -> Result<(), InvoiceError> {
        // Get invoice details
        let detail = self.invoice_repo.get_by_id(user_id, invoice_id).await?;

        // Get client info
        let client = self.client_repo.find_by_id(user_id, detail.client_id)
            .await?
            .ok_or(InvoiceError::ClientNotFound)?;

        // Get user info
        let user = self.user_repo.find_by_id(user_id)
            .await?
            .ok_or(InvoiceError::Validation("User not found".to_string()))?;

        // Get phone number
        let phone = client.phone.clone()
            .ok_or(InvoiceError::Validation("Client has no phone number".to_string()))?;

        // Get guest token
        let guest_token = detail.guest_payment_token.clone()
            .ok_or(InvoiceError::Validation("Invoice has no guest token".to_string()))?;

        // Send WhatsApp notification
        let payment_link = Some(format!("https://yourapp.com/guest/pay/{}", guest_token));
        self.whatsapp_service.send_invoice(
            &phone,
            &detail,
            &user,
            payment_link,
        ).await?;

        // Update tracking
        self.invoice_repo.update_notification_sent(invoice_id, true).await?;

        Ok(())
    }

    /// Send payment confirmation notification
    pub async fn send_payment_confirmation(
        &self,
        user_id: Uuid,
        invoice_id: Uuid,
    ) -> Result<(), InvoiceError> {
        let detail = self.invoice_repo.get_by_id(user_id, invoice_id).await?;
        let client = self.client_repo.find_by_id(user_id, detail.client_id)
            .await?
            .ok_or(InvoiceError::ClientNotFound)?;

        // Send email confirmation
        if let Some(email) = client.email.clone() {
            let _ = self.notification_service.send_payment_confirmation(
                &detail,
                Some(email),
                client.phone.clone(),
            ).await;
        }

        Ok(())
    }

    /// Mark invoice as viewed (for read receipt tracking)
    pub async fn mark_as_viewed(
        &self,
        invoice_id: Uuid,
    ) -> Result<InvoiceDetailResponse, InvoiceError> {
        // Update in repository
        self.invoice_repo.mark_as_viewed(invoice_id).await?;

        // Get updated details
        // Note: We need user_id, so we first get the invoice to get user_id
        let invoice = self.invoice_repo.get_invoice_by_id(invoice_id).await?;
        let detail = self.invoice_repo.get_by_id(invoice.user_id, invoice_id).await?;

        Ok(detail)
    }

    /// Send unviewed reminder notifications
    pub async fn send_unviewed_reminders(&self) -> Result<usize, InvoiceError> {
        // Get invoices that haven't been viewed within time threshold
        // This would typically be called by a background job
        // For now, we'll just return 0 as a placeholder
        // Real implementation would query for invoices sent > 24 hours ago, not viewed
        Ok(0)
    }

    pub async fn get_invoice_pdf(
        &self,
        user_id: Uuid,
        invoice_id: Uuid,
    ) -> Result<Vec<u8>, InvoiceError> {
        // Get invoice details
        let detail = self.invoice_repo.get_by_id(user_id, invoice_id).await?;

        // Get user (company) info
        let user = self.user_repo.find_by_id(user_id)
            .await?
            .ok_or(InvoiceError::Validation("User not found".to_string()))?;

        // Get client info
        let client = self.client_repo.find_by_id(user_id, detail.client_id)
            .await?
            .ok_or(InvoiceError::ClientNotFound)?;

        // Generate PDF
        let items: Vec<InvoiceItemPdf> = detail.items.iter().map(|item| InvoiceItemPdf {
            description: item.description.clone(),
            quantity: item.quantity,
            unit_price: item.unit_price,
            total: item.total,
        }).collect();

        let company_address = user.business_address.as_ref().map(|addr| {
            format!("{}, {}, {} {}", addr.street, addr.city, addr.state, addr.zip_code)
        });

        let client_address = client.billing_address.as_ref().map(|addr| {
            addr.to_string()
        });

        let pdf_bytes = self.pdf_service.generate_invoice_pdf(
            &detail.invoice_number,
            user.company_name.as_deref(),
            company_address.as_deref(),
            &client.name,
            client.email.as_deref(),
            client_address.as_deref(),
            &detail.issue_date.to_string(),
            &detail.due_date.to_string(),
            &items,
            detail.subtotal,
            detail.tax_amount,
            detail.discount_amount,
            detail.total_amount,
            detail.notes.as_deref(),
            detail.terms.as_deref(),
            detail.tax_label.as_deref(),
        )?;

        Ok(pdf_bytes)
    }

    /// Send a payment reminder for an invoice
    pub async fn send_reminder(
        &self,
        user_id: Uuid,
        invoice_id: Uuid,
    ) -> Result<(), InvoiceError> {
        // Get invoice details
        let detail = self.invoice_repo.get_by_id(user_id, invoice_id).await?;

        // Get user (company) info
        let user = self.user_repo.find_by_id(user_id)
            .await?
            .ok_or(InvoiceError::Validation("User not found".to_string()))?;

        // Get client info
        let client = self.client_repo.find_by_id(user_id, detail.client_id)
            .await?
            .ok_or(InvoiceError::ClientNotFound)?;

        // Calculate days overdue
        let today = chrono::Utc::now().naive_utc().date();
        let days_overdue = today.signed_duration_since(detail.due_date).num_days();

        if days_overdue < 0 {
            return Err(InvoiceError::Validation("Invoice is not yet due".to_string()));
        }

        // Determine reminder type based on days overdue
        let (_reminder_type, subject, message) = if days_overdue == 0 {
            ("friendly", "Friendly Reminder: Invoice Due Today", "Just a friendly reminder that your invoice is due today.")
        } else if days_overdue <= 7 {
            ("reminder", "Payment Reminder", "This is a reminder that your invoice is overdue.")
        } else if days_overdue <= 30 {
            ("urgent", "Urgent: Payment Overdue", "Your invoice is significantly overdue. Please remit payment immediately.")
        } else {
            ("final", "Final Notice", "This is our final notice. Immediate payment is required to avoid further action.")
        };

        // Send email reminder
        let subject = format!("[{}] {}", user.company_name.clone().unwrap_or_default(), subject);
        let html_body = format!(
            r#"
            <html>
            <body style="font-family: Arial, sans-serif; padding: 20px;">
                <h2>{}</h2>
                <p>{}</p>
                <p><strong>Invoice #:</strong> {}</p>
                <p><strong>Amount Due:</strong> ${:.2}</p>
                <p><strong>Due Date:</strong> {}</p>
                <p><strong>Days Overdue:</strong> {}</p>
                <p>Please remit payment at your earliest convenience.</p>
                <hr>
                <p style="color: #666;">This is an automated message from {}</p>
            </body>
            </html>
            "#,
            subject,
            message,
            detail.invoice_number,
            detail.total_amount,
            detail.due_date,
            days_overdue,
            user.company_name.clone().unwrap_or_else(|| "FlashBill".to_string())
        );

        self.email_service
            .send_email(
                &client.email.clone().ok_or(InvoiceError::Validation("Client email required".to_string()))?,
                &client.name,
                &subject,
                &html_body,
            )
            .map_err(|e| InvoiceError::EmailError(e.to_string()))?;

        // Update reminder tracking
        self.invoice_repo
            .update_reminder_sent_count(user_id, invoice_id, detail.reminder_sent_count + 1)
            .await?;

        Ok(())
    }

    /// Add a discussion message to an invoice
    pub async fn add_discussion_message(
        &self,
        user_id: Uuid,
        invoice_id: Uuid,
        sender_type: SenderType,
        message: String,
    ) -> Result<InvoiceDiscussion, InvoiceError> {
        // Verify user owns the invoice
        let _invoice = self.invoice_repo.get_by_id(user_id, invoice_id).await?;

        // Add message via repository
        let discussion = self.invoice_repo
            .add_discussion_message(invoice_id, sender_type, message)
            .await?;

        Ok(discussion)
    }

    /// Add a discussion message as a guest (buyer)
    pub async fn add_discussion_message_guest(
        &self,
        invoice_id: Uuid,
        message: String,
    ) -> Result<InvoiceDiscussion, InvoiceError> {
        let discussion = self.invoice_repo
            .add_discussion_message_guest(invoice_id, message)
            .await?;

        Ok(discussion)
    }

    /// Get all discussion messages for an invoice (seller access)
    pub async fn get_discussion_messages(
        &self,
        user_id: Uuid,
        invoice_id: Uuid,
    ) -> Result<Vec<DiscussionResponse>, InvoiceError> {
        // Verify user owns the invoice
        let _invoice = self.invoice_repo.get_by_id(user_id, invoice_id).await?;

        let messages = self.invoice_repo
            .get_discussion_messages(invoice_id)
            .await?;

        Ok(messages)
    }

    /// Get all discussion messages for an invoice (guest access)
    pub async fn get_discussion_messages_guest(
        &self,
        invoice_id: Uuid,
    ) -> Result<Vec<DiscussionResponse>, InvoiceError> {
        let messages = self.invoice_repo
            .get_discussion_messages_guest(invoice_id)
            .await?;

        Ok(messages)
    }
}
