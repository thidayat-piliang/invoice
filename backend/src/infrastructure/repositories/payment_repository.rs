use sqlx::{PgPool, QueryBuilder, Postgres, Row};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::domain::models::{Payment, PaymentResponse, PaymentStats, PaymentStatus, PaymentMethod};

#[derive(Clone)]
pub struct PaymentRepository {
    db: PgPool,
}

impl PaymentRepository {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn create(
        &self,
        user_id: Uuid,
        invoice_id: Uuid,
        amount: f64,
        currency: String,
        payment_method: PaymentMethod,
        gateway: Option<String>,
        gateway_payment_id: Option<String>,
        gateway_fee: Option<f64>,
        paid_by: Option<String>,
        notes: Option<String>,
    ) -> Result<Payment, sqlx::Error> {
        let payment = sqlx::query_as::<_, PaymentRow>(
            r#"
            INSERT INTO payments (
                id, invoice_id, user_id, amount, currency, payment_method,
                gateway, gateway_payment_id, gateway_fee, status, paid_by, notes,
                created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(invoice_id)
        .bind(user_id)
        .bind(amount)
        .bind(currency)
        .bind(payment_method.to_string())
        .bind(&gateway)
        .bind(&gateway_payment_id)
        .bind(gateway_fee.unwrap_or(0.0))
        .bind("completed") // status
        .bind(&paid_by)
        .bind(notes)
        .bind(Utc::now())
        .bind(Utc::now())
        .fetch_one(&self.db)
        .await?;

        Ok(payment.to_payment())
    }

    pub async fn find_by_id(&self, user_id: Uuid, payment_id: Uuid) -> Result<Option<PaymentResponse>, sqlx::Error> {
        let payment = sqlx::query_as::<_, PaymentResponseRow>(
            r#"
            SELECT
                p.*,
                i.invoice_number
            FROM payments p
            LEFT JOIN invoices i ON p.invoice_id = i.id
            WHERE p.id = $1 AND p.user_id = $2
            "#,
        )
        .bind(payment_id)
        .bind(user_id)
        .fetch_optional(&self.db)
        .await?;

        Ok(payment.map(|p| p.to_payment_response()))
    }

    pub async fn list(
        &self,
        user_id: Uuid,
        status: Option<PaymentStatus>,
        payment_method: Option<PaymentMethod>,
        date_from: Option<DateTime<Utc>>,
        date_to: Option<DateTime<Utc>>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<PaymentResponse>, sqlx::Error> {
        let mut query_builder = QueryBuilder::<Postgres>::new(
            r#"
            SELECT
                p.*,
                i.invoice_number
            FROM payments p
            LEFT JOIN invoices i ON p.invoice_id = i.id
            WHERE p.user_id = "#
        );

        query_builder.push_bind(user_id);

        if let Some(s) = status {
            query_builder.push(" AND p.status = ");
            query_builder.push_bind(s.to_string());
        }

        if let Some(pm) = payment_method {
            query_builder.push(" AND p.payment_method = ");
            query_builder.push_bind(pm.to_string());
        }

        if let Some(df) = date_from {
            query_builder.push(" AND p.created_at >= ");
            query_builder.push_bind(df);
        }

        if let Some(dt) = date_to {
            query_builder.push(" AND p.created_at <= ");
            query_builder.push_bind(dt);
        }

        query_builder.push(" ORDER BY p.created_at DESC");

        if let Some(l) = limit {
            query_builder.push(" LIMIT ");
            query_builder.push_bind(l);
        }

        if let Some(o) = offset {
            query_builder.push(" OFFSET ");
            query_builder.push_bind(o);
        }

        let payments = query_builder
            .build_query_as()
            .fetch_all(&self.db)
            .await?;

        Ok(payments.into_iter().map(|p: PaymentResponseRow| p.to_payment_response()).collect())
    }

    pub async fn refund(
        &self,
        user_id: Uuid,
        payment_id: Uuid,
        amount: Option<f64>,
        reason: String,
    ) -> Result<Payment, sqlx::Error> {
        // Get original payment
        let original = sqlx::query_as::<_, PaymentRow>(
            "SELECT * FROM payments WHERE id = $1 AND user_id = $2"
        )
        .bind(payment_id)
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        // Create refund record
        let refund_amount = amount.unwrap_or(original.amount);

        let refund = sqlx::query_as::<_, PaymentRow>(
            r#"
            INSERT INTO payments (
                id, invoice_id, user_id, amount, currency, payment_method,
                gateway, gateway_payment_id, gateway_fee, status, paid_by, notes,
                created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(original.invoice_id)
        .bind(user_id)
        .bind(-refund_amount) // Negative amount for refund
        .bind(&original.currency)
        .bind(original.payment_method)
        .bind(&original.gateway)
        .bind(&original.gateway_payment_id)
        .bind(0.0)
        .bind("refunded")
        .bind(&original.paid_by)
        .bind(Some(format!("Refund: {}", reason)))
        .bind(Utc::now())
        .bind(Utc::now())
        .fetch_one(&self.db)
        .await?;

        Ok(refund.to_payment())
    }

    pub async fn get_stats(&self, user_id: Uuid) -> Result<PaymentStats, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT
                COUNT(*) as total_payments,
                COALESCE(SUM(amount)::float8, 0.0::float8) as total_amount,
                COALESCE(AVG(amount)::float8, 0.0::float8) as avg_payment,
                COALESCE(SUM(CASE WHEN payment_method = 'stripe' THEN amount END)::float8, 0.0::float8) as stripe_total,
                COALESCE(SUM(CASE WHEN payment_method = 'paypal' THEN amount END)::float8, 0.0::float8) as paypal_total,
                COALESCE(SUM(CASE WHEN payment_method = 'check' THEN amount END)::float8, 0.0::float8) as check_total,
                COALESCE(SUM(CASE WHEN payment_method = 'cash' THEN amount END)::float8, 0.0::float8) as cash_total,
                COALESCE(SUM(CASE WHEN payment_method = 'bank_transfer' THEN amount END)::float8, 0.0::float8) as bank_transfer_total
            FROM payments
            WHERE user_id = $1 AND status = 'completed'
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        let mut by_method = std::collections::HashMap::new();
        by_method.insert("stripe".to_string(), row.get::<f64, _>("stripe_total"));
        by_method.insert("paypal".to_string(), row.get::<f64, _>("paypal_total"));
        by_method.insert("check".to_string(), row.get::<f64, _>("check_total"));
        by_method.insert("cash".to_string(), row.get::<f64, _>("cash_total"));
        by_method.insert("bank_transfer".to_string(), row.get::<f64, _>("bank_transfer_total"));

        Ok(PaymentStats {
            total_payments: row.get("total_payments"),
            total_amount: row.get("total_amount"),
            avg_payment: row.get("avg_payment"),
            by_method,
        })
    }
}

#[derive(sqlx::FromRow)]
struct PaymentRow {
    id: Uuid,
    invoice_id: Uuid,
    user_id: Uuid,
    amount: f64,
    currency: String,
    payment_method: String,
    gateway: Option<String>,
    gateway_payment_id: Option<String>,
    gateway_fee: f64,
    status: String,
    failure_reason: Option<String>,
    paid_by: Option<String>,
    notes: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl PaymentRow {
    fn to_payment(self) -> Payment {
        let payment_method = match self.payment_method.as_str() {
            "paypal" => PaymentMethod::PayPal,
            "check" => PaymentMethod::Check,
            "cash" => PaymentMethod::Cash,
            "bank_transfer" => PaymentMethod::BankTransfer,
            _ => PaymentMethod::Stripe,
        };

        let status = match self.status.as_str() {
            "pending" => PaymentStatus::Pending,
            "completed" => PaymentStatus::Completed,
            "failed" => PaymentStatus::Failed,
            "refunded" => PaymentStatus::Refunded,
            _ => PaymentStatus::Pending,
        };

        Payment {
            id: self.id,
            invoice_id: self.invoice_id,
            user_id: self.user_id,
            amount: self.amount,
            currency: self.currency,
            payment_method,
            gateway: self.gateway,
            gateway_payment_id: self.gateway_payment_id,
            gateway_fee: self.gateway_fee,
            status,
            failure_reason: self.failure_reason,
            paid_by: self.paid_by,
            notes: self.notes,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct PaymentResponseRow {
    id: Uuid,
    invoice_id: Uuid,
    invoice_number: Option<String>,
    amount: f64,
    currency: String,
    payment_method: String,
    status: String,
    paid_by: Option<String>,
    notes: Option<String>,
    created_at: DateTime<Utc>,
}

impl PaymentResponseRow {
    fn to_payment_response(self) -> PaymentResponse {
        let payment_method = match self.payment_method.as_str() {
            "paypal" => PaymentMethod::PayPal,
            "check" => PaymentMethod::Check,
            "cash" => PaymentMethod::Cash,
            "bank_transfer" => PaymentMethod::BankTransfer,
            _ => PaymentMethod::Stripe,
        };

        let status = match self.status.as_str() {
            "pending" => PaymentStatus::Pending,
            "completed" => PaymentStatus::Completed,
            "failed" => PaymentStatus::Failed,
            "refunded" => PaymentStatus::Refunded,
            _ => PaymentStatus::Pending,
        };

        PaymentResponse {
            id: self.id,
            invoice_id: self.invoice_id,
            invoice_number: self.invoice_number,
            amount: self.amount,
            currency: self.currency,
            payment_method,
            status,
            paid_by: self.paid_by,
            notes: self.notes,
            created_at: self.created_at,
        }
    }
}
