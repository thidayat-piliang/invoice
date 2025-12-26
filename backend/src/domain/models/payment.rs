use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Type;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(type_name = "varchar")]
pub enum PaymentStatus {
    Pending,
    Completed,
    Failed,
    Refunded,
}

impl std::fmt::Display for PaymentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PaymentStatus::Pending => write!(f, "pending"),
            PaymentStatus::Completed => write!(f, "completed"),
            PaymentStatus::Failed => write!(f, "failed"),
            PaymentStatus::Refunded => write!(f, "refunded"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(type_name = "varchar")]
pub enum PaymentMethod {
    Stripe,
    PayPal,
    Check,
    Cash,
    BankTransfer,
}

impl std::fmt::Display for PaymentMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PaymentMethod::Stripe => write!(f, "stripe"),
            PaymentMethod::PayPal => write!(f, "paypal"),
            PaymentMethod::Check => write!(f, "check"),
            PaymentMethod::Cash => write!(f, "cash"),
            PaymentMethod::BankTransfer => write!(f, "bank_transfer"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Payment {
    pub id: Uuid,
    pub invoice_id: Uuid,
    pub user_id: Uuid,

    #[validate(range(min = 0.01))]
    pub amount: f64,

    pub currency: String,
    pub payment_method: PaymentMethod,

    pub gateway: Option<String>,
    pub gateway_payment_id: Option<String>,
    pub gateway_fee: f64,

    pub status: PaymentStatus,
    pub failure_reason: Option<String>,

    pub paid_by: Option<String>,
    pub notes: Option<String>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreatePayment {
    pub invoice_id: Uuid,

    #[validate(range(min = 0.01))]
    pub amount: f64,

    pub payment_method: PaymentMethod,
    pub gateway: Option<String>,
    pub gateway_payment_id: Option<String>,
    pub gateway_fee: Option<f64>,
    pub paid_by: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentResponse {
    pub id: Uuid,
    pub invoice_id: Uuid,
    pub invoice_number: Option<String>,
    pub amount: f64,
    pub currency: String,
    pub payment_method: PaymentMethod,
    pub status: PaymentStatus,
    pub paid_by: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentListFilter {
    pub status: Option<PaymentStatus>,
    pub payment_method: Option<PaymentMethod>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentStats {
    pub total_payments: i64,
    pub total_amount: f64,
    pub avg_payment: f64,
    pub by_method: std::collections::HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefundRequest {
    pub amount: Option<f64>,
    pub reason: String,
}
