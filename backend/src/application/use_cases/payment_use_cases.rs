use std::sync::Arc;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use thiserror::Error;

use crate::domain::services::PaymentService;
use crate::domain::models::{Payment, PaymentResponse, PaymentStats, PaymentStatus, PaymentMethod, CreatePayment, RefundRequest};

#[derive(Debug, Error)]
pub enum PaymentError {
    #[error("Payment not found")]
    NotFound,
    #[error("Database error: {0}")]
    DatabaseError(String),
}

impl From<sqlx::Error> for PaymentError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => PaymentError::NotFound,
            _ => PaymentError::DatabaseError(err.to_string()),
        }
    }
}

// CreatePaymentUseCase
#[derive(Clone)]
pub struct CreatePaymentUseCase {
    payment_service: Arc<PaymentService>,
}

impl CreatePaymentUseCase {
    pub fn new(payment_service: Arc<PaymentService>) -> Self {
        Self { payment_service }
    }

    pub async fn execute(&self, user_id: Uuid, create: CreatePayment) -> Result<Payment, PaymentError> {
        Ok(self.payment_service.create_payment(user_id, create).await?)
    }
}

// GetPaymentUseCase
#[derive(Clone)]
pub struct GetPaymentUseCase {
    payment_service: Arc<PaymentService>,
}

impl GetPaymentUseCase {
    pub fn new(payment_service: Arc<PaymentService>) -> Self {
        Self { payment_service }
    }

    pub async fn execute(&self, user_id: Uuid, payment_id: Uuid) -> Result<PaymentResponse, PaymentError> {
        let payment = self.payment_service.get_payment(user_id, payment_id).await?;
        payment.ok_or(PaymentError::NotFound)
    }
}

// ListPaymentsUseCase
#[derive(Clone)]
pub struct ListPaymentsUseCase {
    payment_service: Arc<PaymentService>,
}

impl ListPaymentsUseCase {
    pub fn new(payment_service: Arc<PaymentService>) -> Self {
        Self { payment_service }
    }

    pub async fn execute(
        &self,
        user_id: Uuid,
        status: Option<PaymentStatus>,
        payment_method: Option<PaymentMethod>,
        date_from: Option<DateTime<Utc>>,
        date_to: Option<DateTime<Utc>>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<PaymentResponse>, PaymentError> {
        Ok(self.payment_service.list_payments(
            user_id, status, payment_method, date_from, date_to, limit, offset
        ).await?)
    }
}

// RefundPaymentUseCase
#[derive(Clone)]
pub struct RefundPaymentUseCase {
    payment_service: Arc<PaymentService>,
}

impl RefundPaymentUseCase {
    pub fn new(payment_service: Arc<PaymentService>) -> Self {
        Self { payment_service }
    }

    pub async fn execute(
        &self,
        user_id: Uuid,
        payment_id: Uuid,
        refund: RefundRequest,
    ) -> Result<Payment, PaymentError> {
        Ok(self.payment_service.refund_payment(user_id, payment_id, refund).await?)
    }
}

// GetPaymentStatsUseCase
#[derive(Clone)]
pub struct GetPaymentStatsUseCase {
    payment_service: Arc<PaymentService>,
}

impl GetPaymentStatsUseCase {
    pub fn new(payment_service: Arc<PaymentService>) -> Self {
        Self { payment_service }
    }

    pub async fn execute(&self, user_id: Uuid) -> Result<PaymentStats, PaymentError> {
        Ok(self.payment_service.get_stats(user_id).await?)
    }
}

// GetPaymentMethodsUseCase (static list)
#[derive(Clone)]
pub struct GetPaymentMethodsUseCase;

impl GetPaymentMethodsUseCase {
    pub fn new() -> Self {
        Self
    }

    pub fn execute(&self) -> Vec<String> {
        vec![
            "stripe".to_string(),
            "paypal".to_string(),
            "check".to_string(),
            "cash".to_string(),
            "bank_transfer".to_string(),
        ]
    }
}
