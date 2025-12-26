use std::sync::Arc;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::infrastructure::repositories::{PaymentRepository, InvoiceRepository, ClientRepository, UserRepository};
use crate::domain::services::EmailService;
use crate::domain::models::{Payment, PaymentResponse, PaymentStats, PaymentStatus, PaymentMethod, CreatePayment, RefundRequest};

#[derive(Clone)]
pub struct PaymentService {
    payment_repo: Arc<PaymentRepository>,
    invoice_repo: Arc<InvoiceRepository>,
    client_repo: Arc<ClientRepository>,
    user_repo: Arc<UserRepository>,
    email_service: Arc<EmailService>,
}

impl PaymentService {
    pub fn new(
        payment_repo: Arc<PaymentRepository>,
        invoice_repo: Arc<InvoiceRepository>,
        client_repo: Arc<ClientRepository>,
        user_repo: Arc<UserRepository>,
        email_service: Arc<EmailService>,
    ) -> Self {
        Self {
            payment_repo,
            invoice_repo,
            client_repo,
            user_repo,
            email_service
        }
    }

    pub async fn create_payment(
        &self,
        user_id: Uuid,
        create: CreatePayment,
    ) -> Result<Payment, sqlx::Error> {
        let payment = self.payment_repo.create(
            user_id,
            create.invoice_id,
            create.amount,
            "USD".to_string(),
            create.payment_method,
            create.gateway,
            create.gateway_payment_id,
            create.gateway_fee,
            create.paid_by,
            create.notes,
        ).await?;

        // Send payment confirmation email (best effort - don't fail the payment if email fails)
        let _ = self.send_payment_confirmation_email(user_id, create.invoice_id, &payment).await;

        Ok(payment)
    }

    async fn send_payment_confirmation_email(
        &self,
        user_id: Uuid,
        invoice_id: Uuid,
        payment: &Payment,
    ) -> Result<(), sqlx::Error> {
        // Fetch invoice (returns InvoiceDetailResponse which includes client info)
        let invoice = match self.invoice_repo.get_by_id(user_id, invoice_id).await {
            Ok(inv) => inv,
            Err(sqlx::Error::RowNotFound) => return Ok(()),
            Err(e) => return Err(e),
        };

        // Fetch user for notification settings
        let user = match self.user_repo.find_by_id(user_id).await? {
            Some(u) => u,
            None => return Ok(()),
        };

        // Check if user wants payment confirmation emails
        if !user.notification_settings.email_payment_received {
            return Ok(());
        }

        // Get client email from invoice (InvoiceDetailResponse has client_email field)
        let client_email = invoice.client_email.as_deref().unwrap_or("");
        if client_email.is_empty() {
            return Ok(());
        }

        // Get client name (String, not Option)
        let client_name = &invoice.client_name;

        // Send payment confirmation
        let payment_method_str = match payment.payment_method {
            PaymentMethod::Stripe => "Stripe",
            PaymentMethod::PayPal => "PayPal",
            PaymentMethod::Check => "Check",
            PaymentMethod::Cash => "Cash",
            PaymentMethod::BankTransfer => "Bank Transfer",
        };

        let _ = self.email_service.send_payment_confirmation(
            client_email,
            client_name,
            &invoice.invoice_number,
            payment.amount,
            payment_method_str,
        );

        Ok(())
    }

    pub async fn get_payment(
        &self,
        user_id: Uuid,
        payment_id: Uuid,
    ) -> Result<Option<PaymentResponse>, sqlx::Error> {
        self.payment_repo.find_by_id(user_id, payment_id).await
    }

    pub async fn list_payments(
        &self,
        user_id: Uuid,
        status: Option<PaymentStatus>,
        payment_method: Option<PaymentMethod>,
        date_from: Option<DateTime<Utc>>,
        date_to: Option<DateTime<Utc>>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<PaymentResponse>, sqlx::Error> {
        self.payment_repo.list(user_id, status, payment_method, date_from, date_to, limit, offset).await
    }

    pub async fn refund_payment(
        &self,
        user_id: Uuid,
        payment_id: Uuid,
        refund: RefundRequest,
    ) -> Result<Payment, sqlx::Error> {
        self.payment_repo.refund(user_id, payment_id, refund.amount, refund.reason).await
    }

    pub async fn get_stats(&self, user_id: Uuid) -> Result<PaymentStats, sqlx::Error> {
        self.payment_repo.get_stats(user_id).await
    }
}
