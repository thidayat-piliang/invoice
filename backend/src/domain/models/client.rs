use chrono::{DateTime, Utc};
use sqlx::{FromRow, Row};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Client {
    pub id: Uuid,
    pub user_id: Uuid,

    #[validate(length(min = 1, max = 255))]
    pub name: String,

    #[validate(email)]
    pub email: Option<String>,

    pub phone: Option<String>,
    pub company_name: Option<String>,

    pub billing_address: Option<serde_json::Value>,

    pub payment_terms: i32, // days
    pub tax_exempt: bool,
    pub tax_exempt_certificate: Option<String>,
    pub notes: Option<String>,

    pub total_invoiced: f64,
    pub total_paid: f64,
    pub average_payment_days: Option<i32>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateClient {
    #[validate(length(min = 1, max = 255))]
    pub name: String,

    #[validate(email)]
    pub email: Option<String>,

    pub phone: Option<String>,
    pub company_name: Option<String>,

    pub billing_address: Option<serde_json::Value>,

    pub payment_terms: Option<i32>,
    pub tax_exempt: Option<bool>,
    pub tax_exempt_certificate: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateClient {
    pub name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub company_name: Option<String>,
    pub billing_address: Option<serde_json::Value>,
    pub payment_terms: Option<i32>,
    pub tax_exempt: Option<bool>,
    pub tax_exempt_certificate: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientListFilter {
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientResponse {
    pub id: Uuid,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub company_name: Option<String>,
    pub total_invoiced: f64,
    pub total_paid: f64,
    pub outstanding_balance: f64,
    pub average_payment_days: Option<i32>,
    pub last_invoice_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl FromRow<'_, sqlx::postgres::PgRow> for ClientResponse {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        Ok(ClientResponse {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            email: row.try_get("email")?,
            phone: row.try_get("phone")?,
            company_name: row.try_get("company_name")?,
            total_invoiced: row.try_get("total_invoiced")?,
            total_paid: row.try_get("total_paid")?,
            outstanding_balance: row.try_get("outstanding_balance")?,
            average_payment_days: row.try_get("average_payment_days")?,
            last_invoice_date: row.try_get("last_invoice_date")?,
            created_at: row.try_get("created_at")?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientStats {
    pub total_clients: i64,
    pub active_clients: i64,
    pub total_invoiced: f64,
    pub total_paid: f64,
    pub outstanding_balance: f64,
    pub avg_payment_days: f64,
}
