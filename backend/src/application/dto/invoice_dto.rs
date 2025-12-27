use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::models::{InvoiceStatus, InvoiceItem};

// Input DTOs (from API layer)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateInvoiceCommand {
    pub client_id: Uuid,
    pub issue_date: NaiveDate,
    pub due_date: NaiveDate,
    pub items: Vec<CreateInvoiceItemCommand>,
    pub notes: Option<String>,
    pub terms: Option<String>,
    pub discount_amount: Option<f64>,
    pub tax_included: bool,
    pub send_immediately: bool,
    pub allow_partial_payment: Option<bool>,
    pub min_payment_amount: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateInvoiceItemCommand {
    pub description: String,
    pub quantity: f64,
    pub unit_price: f64,
    pub tax_rate: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInvoiceCommand {
    pub client_id: Option<Uuid>,
    pub issue_date: Option<NaiveDate>,
    pub due_date: Option<NaiveDate>,
    pub items: Option<Vec<CreateInvoiceItemCommand>>,
    pub notes: Option<String>,
    pub terms: Option<String>,
    pub discount_amount: Option<f64>,
    pub tax_included: Option<bool>,
    pub allow_partial_payment: Option<bool>,
    pub min_payment_amount: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordPaymentCommand {
    pub amount: f64,
    pub payment_method: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendInvoiceCommand {
    pub email: Option<String>,
    pub subject: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceListQuery {
    pub status: Option<String>,
    pub client_id: Option<Uuid>,
    pub date_from: Option<NaiveDate>,
    pub date_to: Option<NaiveDate>,
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

// Output DTOs (to API layer)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceDto {
    pub id: Uuid,
    pub invoice_number: String,
    pub status: InvoiceStatus,
    pub client_id: Uuid,
    pub client_name: String,
    pub client_email: Option<String>,
    pub client_phone: Option<String>,
    pub client_address: Option<serde_json::Value>,
    pub issue_date: NaiveDate,
    pub due_date: NaiveDate,
    pub subtotal: f64,
    pub tax_amount: f64,
    pub discount_amount: f64,
    pub total_amount: f64,
    pub amount_paid: f64,
    pub balance_due: f64,
    pub items: Vec<InvoiceItem>,
    pub notes: Option<String>,
    pub terms: Option<String>,
    pub tax_calculation: serde_json::Value,
    pub tax_included: bool,
    pub tax_label: Option<String>,
    pub tax_id: Option<String>,
    pub pdf_url: Option<String>,
    pub receipt_image_url: Option<String>,
    pub sent_at: Option<DateTime<Utc>>,
    pub viewed_at: Option<DateTime<Utc>>,
    pub paid_at: Option<DateTime<Utc>>,
    pub reminder_sent_count: i32,
    pub last_reminder_sent: Option<DateTime<Utc>>,
    pub notification_sent_at: Option<DateTime<Utc>>,
    pub whatsapp_sent_at: Option<DateTime<Utc>>,
    pub guest_payment_token: Option<String>,
    pub allow_partial_payment: bool,
    pub min_payment_amount: Option<f64>,
    pub partial_payment_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceSummaryDto {
    pub id: Uuid,
    pub invoice_number: String,
    pub status: InvoiceStatus,
    pub client_name: String,
    pub client_email: Option<String>,
    pub issue_date: NaiveDate,
    pub due_date: NaiveDate,
    pub total_amount: f64,
    pub balance_due: f64,
    pub days_until_due: i32,
    pub is_overdue: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceCreatedDto {
    pub id: Uuid,
    pub invoice_number: String,
    pub status: InvoiceStatus,
    pub subtotal: f64,
    pub tax_amount: f64,
    pub total_amount: f64,
    pub tax_label: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentRecordedDto {
    pub invoice_id: Uuid,
    pub invoice_number: String,
    pub amount_paid: f64,
    pub new_balance: f64,
    pub status: InvoiceStatus,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendInvoiceResponse {
    pub success: bool,
    pub email_sent: bool,
    pub whatsapp_sent: bool,
    pub guest_payment_token: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceReadStatusDto {
    pub id: Uuid,
    pub invoice_number: String,
    pub status: InvoiceStatus,
    pub sent_at: Option<DateTime<Utc>>,
    pub viewed_at: Option<DateTime<Utc>>,
    pub read_status: String,
    pub days_to_read: Option<i64>,
    pub client_name: String,
    pub client_email: Option<String>,
    pub client_phone: Option<String>,
    pub notification_sent_at: Option<DateTime<Utc>>,
    pub whatsapp_sent_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuestInvoiceInfoDto {
    pub invoice_number: String,
    pub total_amount: f64,
    pub due_date: NaiveDate,
    pub client_name: String,
    pub client_email: Option<String>,
    pub client_phone: Option<String>,
    pub guest_payment_token: String,
    pub seller_company_name: Option<String>,
    pub seller_email: String,
    pub seller_phone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceNotificationDto {
    pub invoice_id: Uuid,
    pub invoice_number: String,
    pub email_sent: bool,
    pub whatsapp_sent: bool,
    pub notification_sent_at: Option<DateTime<Utc>>,
    pub whatsapp_sent_at: Option<DateTime<Utc>>,
}

// Discussion DTOs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddDiscussionMessageCommand {
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscussionMessageDto {
    pub id: Uuid,
    pub invoice_id: Uuid,
    pub sender_type: String,
    pub message: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscussionResponseDto {
    pub messages: Vec<DiscussionMessageDto>,
}
