use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Type, FromRow, Row};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq)]
#[sqlx(type_name = "varchar")]
pub enum InvoiceStatus {
    Draft,
    Sent,
    Viewed,
    Partial,
    Paid,
    Overdue,
    Cancelled,
}

impl std::fmt::Display for InvoiceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InvoiceStatus::Draft => write!(f, "draft"),
            InvoiceStatus::Sent => write!(f, "sent"),
            InvoiceStatus::Viewed => write!(f, "viewed"),
            InvoiceStatus::Partial => write!(f, "partial"),
            InvoiceStatus::Paid => write!(f, "paid"),
            InvoiceStatus::Overdue => write!(f, "overdue"),
            InvoiceStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct InvoiceItem {
    pub id: Uuid,

    #[validate(length(min = 1, max = 1000))]
    pub description: String,

    #[validate(range(min = 0.01))]
    pub quantity: f64,

    #[validate(range(min = 0.01))]
    pub unit_price: f64,

    #[validate(range(min = 0.0, max = 100.0))]
    pub tax_rate: f64,

    pub tax_amount: f64,
    pub total: f64,
}

impl InvoiceItem {
    pub fn calculate(&mut self) {
        self.tax_amount = (self.quantity * self.unit_price) * (self.tax_rate / 100.0);
        self.total = (self.quantity * self.unit_price) + self.tax_amount;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Invoice {
    pub id: Uuid,
    pub user_id: Uuid,
    pub client_id: Uuid,

    #[validate(length(min = 5, max = 50))]
    pub invoice_number: String,

    pub status: InvoiceStatus,

    pub issue_date: NaiveDate,
    pub due_date: NaiveDate,

    pub subtotal: f64,
    pub tax_amount: f64,
    pub discount_amount: f64,
    pub total_amount: f64,
    pub amount_paid: f64,

    pub items: Vec<InvoiceItem>,
    pub notes: Option<String>,
    pub terms: Option<String>,

    pub tax_calculation: serde_json::Value,
    pub tax_included: bool,

    pub pdf_url: Option<String>,
    pub receipt_image_url: Option<String>,

    pub sent_at: Option<DateTime<Utc>>,
    pub paid_at: Option<DateTime<Utc>>,

    pub reminder_sent_count: i32,
    pub last_reminder_sent: Option<DateTime<Utc>>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Invoice {
    pub fn calculate_totals(&mut self) {
        self.subtotal = self.items.iter()
            .map(|item| item.quantity * item.unit_price)
            .sum();

        self.tax_amount = self.items.iter()
            .map(|item| item.tax_amount)
            .sum();

        self.total_amount = self.subtotal + self.tax_amount - self.discount_amount;
    }

    pub fn balance_due(&self) -> f64 {
        self.total_amount - self.amount_paid
    }

    pub fn is_overdue(&self) -> bool {
        if self.status == InvoiceStatus::Paid || self.status == InvoiceStatus::Cancelled {
            return false;
        }

        let today = chrono::Utc::now().naive_utc().date();
        today > self.due_date
    }

    pub fn update_status(&mut self) {
        if self.is_overdue() {
            self.status = InvoiceStatus::Overdue;
        } else if self.amount_paid > 0.0 && self.amount_paid < self.total_amount {
            self.status = InvoiceStatus::Partial;
        } else if self.amount_paid >= self.total_amount {
            self.status = InvoiceStatus::Paid;
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateInvoice {
    pub client_id: Uuid,

    pub issue_date: NaiveDate,

    pub due_date: NaiveDate,

    #[validate(length(min = 1))]
    pub items: Vec<CreateInvoiceItem>,

    pub notes: Option<String>,
    pub terms: Option<String>,
    pub discount_amount: Option<f64>,
    pub tax_included: bool,
    pub send_immediately: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateInvoiceItem {
    #[validate(length(min = 1, max = 1000))]
    pub description: String,

    #[validate(range(min = 0.01))]
    pub quantity: f64,

    #[validate(range(min = 0.01))]
    pub unit_price: f64,

    pub tax_rate: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateInvoice {
    pub client_id: Option<Uuid>,
    pub issue_date: Option<NaiveDate>,
    pub due_date: Option<NaiveDate>,
    pub items: Option<Vec<CreateInvoiceItem>>,
    pub notes: Option<String>,
    pub terms: Option<String>,
    pub discount_amount: Option<f64>,
    pub tax_included: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceListFilter {
    pub status: Option<InvoiceStatus>,
    pub client_id: Option<Uuid>,
    pub date_from: Option<NaiveDate>,
    pub date_to: Option<NaiveDate>,
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceResponse {
    pub id: Uuid,
    pub invoice_number: String,
    pub status: InvoiceStatus,
    pub client_name: String,
    pub client_email: Option<String>,
    pub issue_date: NaiveDate,
    pub due_date: NaiveDate,
    pub total_amount: f64,
    pub balance_due: f64,
    pub days_until_due: i64,
    pub is_overdue: bool,
    pub created_at: DateTime<Utc>,
}

impl FromRow<'_, sqlx::postgres::PgRow> for InvoiceResponse {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        Ok(InvoiceResponse {
            id: row.try_get("id")?,
            invoice_number: row.try_get("invoice_number")?,
            status: row.try_get("status")?,
            client_name: row.try_get("client_name")?,
            client_email: row.try_get("client_email")?,
            issue_date: row.try_get("issue_date")?,
            due_date: row.try_get("due_date")?,
            total_amount: row.try_get("total_amount")?,
            balance_due: row.try_get("balance_due")?,
            days_until_due: row.try_get("days_until_due")?,
            is_overdue: row.try_get("is_overdue")?,
            created_at: row.try_get("created_at")?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceDetailResponse {
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
    pub pdf_url: Option<String>,
    pub receipt_image_url: Option<String>,
    pub sent_at: Option<DateTime<Utc>>,
    pub paid_at: Option<DateTime<Utc>>,
    pub reminder_sent_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl FromRow<'_, sqlx::postgres::PgRow> for InvoiceDetailResponse {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        let items_json: serde_json::Value = row.try_get("items")?;
        let items: Vec<InvoiceItem> = serde_json::from_value(items_json).unwrap_or_default();

        Ok(InvoiceDetailResponse {
            id: row.try_get("id")?,
            invoice_number: row.try_get("invoice_number")?,
            status: row.try_get("status")?,
            client_id: row.try_get("client_id")?,
            client_name: row.try_get("client_name")?,
            client_email: row.try_get("client_email")?,
            client_phone: row.try_get("client_phone")?,
            client_address: row.try_get("client_address")?,
            issue_date: row.try_get("issue_date")?,
            due_date: row.try_get("due_date")?,
            subtotal: row.try_get("subtotal")?,
            tax_amount: row.try_get("tax_amount")?,
            discount_amount: row.try_get("discount_amount")?,
            total_amount: row.try_get("total_amount")?,
            amount_paid: row.try_get("amount_paid")?,
            balance_due: row.try_get("balance_due")?,
            items,
            notes: row.try_get("notes")?,
            terms: row.try_get("terms")?,
            tax_calculation: row.try_get("tax_calculation")?,
            tax_included: row.try_get("tax_included")?,
            pdf_url: row.try_get("pdf_url")?,
            receipt_image_url: row.try_get("receipt_image_url")?,
            sent_at: row.try_get("sent_at")?,
            paid_at: row.try_get("paid_at")?,
            reminder_sent_count: row.try_get("reminder_sent_count")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendInvoiceRequest {
    pub email: Option<String>,
    pub subject: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceReminder {
    pub invoice_id: Uuid,
    pub days_overdue: i64,
    pub reminder_type: String, // 'friendly', 'reminder', 'urgent', 'final_notice'
    pub send_email: bool,
    pub send_sms: bool,
    pub add_late_fee: bool,
}
