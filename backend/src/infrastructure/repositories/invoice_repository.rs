use sqlx::{PgPool, Postgres, QueryBuilder, Row};
use uuid::Uuid;
use chrono::{DateTime, Utc, NaiveDate, Datelike};
use rand::Rng;
use std::sync::Arc;

use crate::domain::models::{
    Invoice, InvoiceItem, InvoiceStatus, InvoiceResponse, InvoiceDetailResponse,
    CreateInvoice, UpdateInvoice, InvoiceListFilter, CreatePayment, PaymentMethod
};
use crate::domain::services::TaxService;

#[derive(Clone)]
pub struct InvoiceRepository {
    db: PgPool,
    tax_service: Arc<TaxService>,
}

impl InvoiceRepository {
    pub fn new(db: PgPool, tax_service: Arc<TaxService>) -> Self {
        Self { db, tax_service }
    }

    pub async fn create(&self, user_id: Uuid, create: CreateInvoice) -> Result<Invoice, sqlx::Error> {
        // Fetch default tax setting for the organization
        let default_tax = self.tax_service.get_default_tax(user_id).await
            .map_err(|_| sqlx::Error::RowNotFound)?;

        // Calculate items first (needed for both retry and final insert)
        let mut items = Vec::new();
        let mut subtotal = 0.0;
        let mut tax_amount = 0.0;

        for item in create.items {
            // Use item tax rate if provided, otherwise use default tax rate
            let tax_rate = item.tax_rate.unwrap_or_else(|| {
                default_tax.as_ref().map(|t| t.rate).unwrap_or(0.0)
            });

            let item_subtotal = item.quantity * item.unit_price;
            let item_tax = item_subtotal * tax_rate;
            let item_total = item_subtotal + item_tax;

            items.push(InvoiceItem {
                id: Uuid::new_v4(),
                description: item.description,
                quantity: item.quantity,
                unit_price: item.unit_price,
                tax_rate,
                tax_amount: item_tax,
                total: item_total,
            });

            subtotal += item_subtotal;
            tax_amount += item_tax;
        }

        let discount = create.discount_amount.unwrap_or(0.0);
        let total_amount = subtotal + tax_amount - discount;

        let tax_calculation = serde_json::json!({
            "subtotal": subtotal,
            "tax_amount": tax_amount,
            "discount": discount,
            "total": total_amount
        });

        let status = if create.send_immediately {
            InvoiceStatus::Sent
        } else {
            InvoiceStatus::Draft
        };

        let sent_at = if create.send_immediately {
            Some(Utc::now())
        } else {
            None
        };

        // Get tax label and ID from default tax
        let tax_label = default_tax.as_ref().map(|t| t.label.clone());
        let tax_id = default_tax.as_ref().map(|t| t.id.to_string());

        let items_json = serde_json::to_value(&items).unwrap_or(serde_json::Value::Array(vec![]));
        let tax_calculation_json = serde_json::to_value(&tax_calculation).unwrap_or(serde_json::Value::Null);

        // Retry logic for duplicate invoice numbers
        let max_retries = 5;
        for attempt in 0..max_retries {
            // Generate random values BEFORE async operations to avoid Send issues
            let random_suffix: u32 = rand::random();

            let count = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM invoices WHERE user_id = $1 AND created_at >= date_trunc('year', CURRENT_DATE)"
            )
            .bind(user_id)
            .fetch_one(&self.db)
            .await?;

            let year = chrono::Local::now().year();
            let invoice_number = format!("INV-{}-{:04}-{:03}", year, count + 1, random_suffix % 1000);

            let result = sqlx::query_as::<_, InvoiceInsertRow>(
                r#"
                INSERT INTO invoices (
                    id, user_id, client_id, invoice_number, status,
                    issue_date, due_date, subtotal, tax_amount, discount_amount,
                    total_amount, amount_paid, items, notes, terms,
                    tax_calculation, tax_included, tax_label, tax_id,
                    sent_at, created_at, updated_at
                ) VALUES (
                    $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22
                )
                RETURNING *
                "#,
            )
            .bind(Uuid::new_v4())
            .bind(user_id)
            .bind(create.client_id)
            .bind(&invoice_number)
            .bind(&status.to_string())
            .bind(create.issue_date)
            .bind(create.due_date)
            .bind(subtotal)
            .bind(tax_amount)
            .bind(discount)
            .bind(total_amount)
            .bind(0.0) // amount_paid
            .bind(&items_json)
            .bind(&create.notes)
            .bind(&create.terms)
            .bind(&tax_calculation_json)
            .bind(create.tax_included)
            .bind(&tax_label)
            .bind(&tax_id)
            .bind(sent_at)
            .bind(Utc::now())
            .bind(Utc::now())
            .fetch_one(&self.db)
            .await;

            match result {
                Ok(invoice) => return Ok(invoice.to_invoice()),
                Err(sqlx::Error::Database(db_err)) => {
                    if db_err.is_unique_violation() && attempt < max_retries - 1 {
                        // Wait a tiny bit and retry
                        tokio::time::sleep(tokio::time::Duration::from_millis(10 * (attempt as u64 + 1))).await;
                        continue;
                    } else {
                        return Err(sqlx::Error::Database(db_err));
                    }
                }
                Err(e) => return Err(e),
            }
        }

        // Fallback: use timestamp-based invoice number with microseconds and random
        let timestamp = chrono::Local::now().format("%Y%m%d%H%M%S%.6f");
        let random_suffix: u32 = rand::random();
        let invoice_number = format!("INV-{}-{:05}", timestamp, random_suffix % 100000);

        let invoice = sqlx::query_as::<_, InvoiceInsertRow>(
            r#"
            INSERT INTO invoices (
                id, user_id, client_id, invoice_number, status,
                issue_date, due_date, subtotal, tax_amount, discount_amount,
                total_amount, amount_paid, items, notes, terms,
                tax_calculation, tax_included, tax_label, tax_id,
                sent_at, created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22
            )
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(create.client_id)
        .bind(&invoice_number)
        .bind(&status.to_string())
        .bind(create.issue_date)
        .bind(create.due_date)
        .bind(subtotal)
        .bind(tax_amount)
        .bind(discount)
        .bind(total_amount)
        .bind(0.0) // amount_paid
        .bind(&items_json)
        .bind(&create.notes)
        .bind(&create.terms)
        .bind(&tax_calculation_json)
        .bind(create.tax_included)
        .bind(&tax_label)
        .bind(&tax_id)
        .bind(sent_at)
        .bind(Utc::now())
        .bind(Utc::now())
        .fetch_one(&self.db)
        .await?;

        Ok(invoice.to_invoice())
    }

    pub async fn get_by_id(&self, user_id: Uuid, invoice_id: Uuid) -> Result<InvoiceDetailResponse, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT
                i.id, i.user_id, i.client_id, i.invoice_number, i.status,
                i.issue_date, i.due_date, i.subtotal, i.tax_amount, i.discount_amount,
                i.total_amount, i.amount_paid, i.items, i.notes, i.terms,
                i.tax_calculation, i.tax_included, i.tax_label, i.tax_id,
                i.pdf_url, i.receipt_image_url,
                i.sent_at, i.paid_at, i.reminder_sent_count, i.last_reminder_sent,
                i.created_at, i.updated_at, i.payment_method, i.payment_reference,
                c.name as client_name,
                c.email as client_email,
                c.phone as client_phone,
                c.billing_address as client_address
            FROM invoices i
            JOIN clients c ON i.client_id = c.id
            WHERE i.id = $1 AND i.user_id = $2
            "#,
        )
        .bind(invoice_id)
        .bind(user_id)
        .fetch_optional(&self.db)
        .await?;

        match row {
            Some(r) => {
                let items: Vec<InvoiceItem> = serde_json::from_value(r.try_get("items")?).unwrap_or_default();
                let status_str: String = r.try_get("status")?;
                let status = match status_str.as_str() {
                    "draft" => InvoiceStatus::Draft,
                    "sent" => InvoiceStatus::Sent,
                    "viewed" => InvoiceStatus::Viewed,
                    "partial" => InvoiceStatus::Partial,
                    "paid" => InvoiceStatus::Paid,
                    "overdue" => InvoiceStatus::Overdue,
                    "cancelled" => InvoiceStatus::Cancelled,
                    _ => InvoiceStatus::Draft,
                };
                let balance_due: f64 = r.try_get::<f64, _>("total_amount")? - r.try_get::<f64, _>("amount_paid")?;

                Ok(InvoiceDetailResponse {
                    id: r.try_get("id")?,
                    invoice_number: r.try_get("invoice_number")?,
                    status,
                    client_id: r.try_get("client_id")?,
                    client_name: r.try_get("client_name")?,
                    client_email: r.try_get("client_email")?,
                    client_phone: r.try_get("client_phone")?,
                    client_address: r.try_get("client_address")?,
                    issue_date: r.try_get("issue_date")?,
                    due_date: r.try_get("due_date")?,
                    subtotal: r.try_get("subtotal")?,
                    tax_amount: r.try_get("tax_amount")?,
                    discount_amount: r.try_get("discount_amount")?,
                    total_amount: r.try_get("total_amount")?,
                    amount_paid: r.try_get("amount_paid")?,
                    balance_due,
                    items,
                    notes: r.try_get("notes")?,
                    terms: r.try_get("terms")?,
                    tax_calculation: r.try_get("tax_calculation")?,
                    tax_included: r.try_get("tax_included")?,
                    tax_label: r.try_get("tax_label")?,
                    tax_id: r.try_get("tax_id")?,
                    pdf_url: r.try_get("pdf_url")?,
                    receipt_image_url: r.try_get("receipt_image_url")?,
                    sent_at: r.try_get("sent_at")?,
                    paid_at: r.try_get("paid_at")?,
                    reminder_sent_count: r.try_get("reminder_sent_count")?,
                    last_reminder_sent: r.try_get("last_reminder_sent")?,
                    created_at: r.try_get("created_at")?,
                    updated_at: r.try_get("updated_at")?,
                })
            }
            None => Err(sqlx::Error::RowNotFound),
        }
    }

    pub async fn list(
        &self,
        user_id: Uuid,
        filter: InvoiceListFilter,
    ) -> Result<Vec<InvoiceResponse>, sqlx::Error> {
        let mut query_builder = QueryBuilder::<Postgres>::new(
            r#"
            SELECT
                i.id, i.invoice_number, i.status, c.name as client_name, c.email as client_email,
                i.issue_date, i.due_date, i.total_amount,
                (i.total_amount - i.amount_paid) as balance_due,
                i.created_at,
                (i.due_date - CURRENT_DATE)::int4 as days_until_due,
                (i.due_date < CURRENT_DATE AND i.status != 'paid' AND i.status != 'cancelled') as is_overdue
            FROM invoices i
            JOIN clients c ON i.client_id = c.id
            WHERE i.user_id = "#,
        );

        query_builder.push_bind(user_id);

        if let Some(status) = filter.status {
            query_builder.push(" AND i.status = ");
            query_builder.push_bind(status.to_string());
        }

        if let Some(client_id) = filter.client_id {
            query_builder.push(" AND i.client_id = ");
            query_builder.push_bind(client_id);
        }

        if let Some(date_from) = filter.date_from {
            query_builder.push(" AND i.issue_date >= ");
            query_builder.push_bind(date_from);
        }

        if let Some(date_to) = filter.date_to {
            query_builder.push(" AND i.issue_date <= ");
            query_builder.push_bind(date_to);
        }

        if let Some(search) = filter.search {
            query_builder.push(" AND (c.name ILIKE ");
            query_builder.push_bind(format!("%{}%", search));
            query_builder.push(" OR i.invoice_number ILIKE ");
            query_builder.push_bind(format!("%{}%", search));
            query_builder.push(")");
        }

        query_builder.push(" ORDER BY i.created_at DESC");

        if let Some(limit) = filter.limit {
            query_builder.push(" LIMIT ");
            query_builder.push_bind(limit);
        }

        if let Some(offset) = filter.offset {
            query_builder.push(" OFFSET ");
            query_builder.push_bind(offset);
        }

        let query = query_builder.build();
        let rows = query.fetch_all(&self.db).await?;

        let invoices: Vec<InvoiceResponse> = rows.into_iter().map(|row| {
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
        }).collect::<Result<Vec<_>, sqlx::Error>>()?;

        Ok(invoices)
    }

    pub async fn update(
        &self,
        user_id: Uuid,
        invoice_id: Uuid,
        update: UpdateInvoice,
    ) -> Result<Invoice, sqlx::Error> {
        // Get existing invoice
        let existing = self.get_invoice_internal(user_id, invoice_id).await?;

        // Calculate new values if items changed
        let mut items = existing.items;
        let mut subtotal = existing.subtotal;
        let mut tax_amount = existing.tax_amount;

        if let Some(new_items) = update.items {
            items = Vec::new();
            subtotal = 0.0;
            tax_amount = 0.0;

            for item in new_items {
                let tax_rate = item.tax_rate.unwrap_or(0.0);
                let item_subtotal = item.quantity * item.unit_price;
                let item_tax = item_subtotal * tax_rate;
                let item_total = item_subtotal + item_tax;

                items.push(InvoiceItem {
                    id: Uuid::new_v4(),
                    description: item.description,
                    quantity: item.quantity,
                    unit_price: item.unit_price,
                    tax_rate,
                    tax_amount: item_tax,
                    total: item_total,
                });

                subtotal += item_subtotal;
                tax_amount += item_tax;
            }
        }

        let discount = update.discount_amount.unwrap_or(existing.discount_amount);
        let total_amount = subtotal + tax_amount - discount;

        // Update fields
        let client_id = update.client_id.unwrap_or(existing.client_id);
        let issue_date = update.issue_date.unwrap_or(existing.issue_date);
        let due_date = update.due_date.unwrap_or(existing.due_date);
        let notes = update.notes.unwrap_or(existing.notes.unwrap_or_default());
        let terms = update.terms.unwrap_or(existing.terms.unwrap_or_default());
        let tax_included = update.tax_included.unwrap_or(existing.tax_included);
        let items_json = serde_json::to_value(&items).unwrap_or(serde_json::Value::Array(vec![]));

        let invoice = sqlx::query_as::<_, InvoiceInsertRow>(
            r#"
            UPDATE invoices SET
                client_id = $1, issue_date = $2, due_date = $3, items = $4,
                notes = $5, terms = $6, discount_amount = $7, tax_included = $8,
                subtotal = $9, tax_amount = $10, total_amount = $11, updated_at = $12
            WHERE id = $13 AND user_id = $14
            RETURNING *
            "#,
        )
        .bind(client_id)
        .bind(issue_date)
        .bind(due_date)
        .bind(&items_json)
        .bind(&notes)
        .bind(&terms)
        .bind(discount)
        .bind(tax_included)
        .bind(subtotal)
        .bind(tax_amount)
        .bind(total_amount)
        .bind(Utc::now())
        .bind(invoice_id)
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        Ok(invoice.to_invoice())
    }

    pub async fn delete(&self, user_id: Uuid, invoice_id: Uuid) -> Result<(), sqlx::Error> {
        let result = sqlx::query(
            "DELETE FROM invoices WHERE id = $1 AND user_id = $2"
        )
        .bind(invoice_id)
        .bind(user_id)
        .execute(&self.db)
        .await?;

        if result.rows_affected() == 0 {
            return Err(sqlx::Error::RowNotFound);
        }

        Ok(())
    }

    pub async fn record_payment(
        &self,
        user_id: Uuid,
        invoice_id: Uuid,
        payment: CreatePayment,
    ) -> Result<Invoice, sqlx::Error> {
        // Get invoice
        let mut invoice = self.get_invoice_internal(user_id, invoice_id).await?;

        // Update amount paid
        invoice.amount_paid += payment.amount;

        // Update status
        if invoice.amount_paid >= invoice.total_amount {
            invoice.status = InvoiceStatus::Paid;
            invoice.paid_at = Some(Utc::now());
        } else if invoice.amount_paid > 0.0 {
            invoice.status = InvoiceStatus::Partial;
        }

        invoice.updated_at = Utc::now();

        // Update in database
        let updated = sqlx::query_as::<_, InvoiceInsertRow>(
            r#"
            UPDATE invoices SET
                amount_paid = $1, status = $2, paid_at = $3, updated_at = $4
            WHERE id = $5 AND user_id = $6
            RETURNING *
            "#,
        )
        .bind(invoice.amount_paid)
        .bind(&invoice.status.to_string())
        .bind(invoice.paid_at)
        .bind(invoice.updated_at)
        .bind(invoice_id)
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        // Create payment record
        sqlx::query(
            r#"
            INSERT INTO payments (
                id, invoice_id, user_id, amount, currency, payment_method,
                status, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(invoice_id)
        .bind(user_id)
        .bind(payment.amount)
        .bind("USD")
        .bind(payment.payment_method.to_string())
        .bind("completed")
        .bind(Utc::now())
        .bind(Utc::now())
        .execute(&self.db)
        .await?;

        Ok(updated.to_invoice())
    }

    pub async fn send_invoice(
        &self,
        user_id: Uuid,
        invoice_id: Uuid,
        email: Option<String>,
    ) -> Result<(), sqlx::Error> {
        // Get client email
        let client_email = match email {
            Some(e) => e,
            None => {
                sqlx::query_scalar::<_, String>(
                    "SELECT email FROM clients WHERE id = (SELECT client_id FROM invoices WHERE id = $1 AND user_id = $2)"
                )
                .bind(invoice_id)
                .bind(user_id)
                .fetch_one(&self.db)
                .await?
            }
        };

        // Update status
        sqlx::query(
            r#"
            UPDATE invoices SET
                status = 'sent', sent_at = $1, updated_at = $2
            WHERE id = $3 AND user_id = $4
            "#,
        )
        .bind(Utc::now())
        .bind(Utc::now())
        .bind(invoice_id)
        .bind(user_id)
        .execute(&self.db)
        .await?;

        // Note: Actual email sending happens in service layer
        // This just updates the database

        Ok(())
    }

    // Internal helper
    async fn get_invoice_internal(&self, user_id: Uuid, invoice_id: Uuid) -> Result<Invoice, sqlx::Error> {
        let row = sqlx::query_as::<_, InvoiceInsertRow>(
            "SELECT * FROM invoices WHERE id = $1 AND user_id = $2"
        )
        .bind(invoice_id)
        .bind(user_id)
        .fetch_optional(&self.db)
        .await?;

        match row {
            Some(r) => Ok(r.to_invoice()),
            None => Err(sqlx::Error::RowNotFound),
        }
    }

    pub async fn update_reminder_sent_count(
        &self,
        user_id: Uuid,
        invoice_id: Uuid,
        count: i32,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE invoices SET
                reminder_sent_count = $1,
                last_reminder_sent = $2,
                updated_at = $3
            WHERE id = $4 AND user_id = $5
            "#,
        )
        .bind(count)
        .bind(Utc::now())
        .bind(Utc::now())
        .bind(invoice_id)
        .bind(user_id)
        .execute(&self.db)
        .await?;

        Ok(())
    }
}

// Helper struct for database query (includes client info)
#[derive(sqlx::FromRow)]
struct InvoiceRow {
    id: Uuid,
    user_id: Uuid,
    client_id: Uuid,
    invoice_number: String,
    status: String,
    issue_date: NaiveDate,
    due_date: NaiveDate,
    subtotal: f64,
    tax_amount: f64,
    discount_amount: f64,
    total_amount: f64,
    amount_paid: f64,
    items: serde_json::Value,
    notes: Option<String>,
    terms: Option<String>,
    tax_calculation: serde_json::Value,
    tax_included: bool,
    tax_label: Option<String>,
    tax_id: Option<String>,
    pdf_url: Option<String>,
    receipt_image_url: Option<String>,
    sent_at: Option<DateTime<Utc>>,
    paid_at: Option<DateTime<Utc>>,
    reminder_sent_count: i32,
    last_reminder_sent: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    // Additional columns in invoices table
    payment_method: Option<String>,
    payment_reference: Option<String>,
    // For detail query
    client_name: Option<String>,
    client_email: Option<String>,
    client_phone: Option<String>,
    client_address: Option<serde_json::Value>,
    // For list query
    days_until_due: Option<i32>,
    is_overdue: Option<bool>,
}

// Helper struct for INSERT RETURNING (only invoice columns)
#[derive(sqlx::FromRow)]
struct InvoiceInsertRow {
    id: Uuid,
    user_id: Uuid,
    client_id: Uuid,
    invoice_number: String,
    status: String,
    issue_date: NaiveDate,
    due_date: NaiveDate,
    subtotal: f64,
    tax_amount: f64,
    discount_amount: f64,
    total_amount: f64,
    amount_paid: f64,
    items: serde_json::Value,
    notes: Option<String>,
    terms: Option<String>,
    tax_calculation: serde_json::Value,
    tax_included: bool,
    tax_label: Option<String>,
    tax_id: Option<String>,
    pdf_url: Option<String>,
    receipt_image_url: Option<String>,
    sent_at: Option<DateTime<Utc>>,
    paid_at: Option<DateTime<Utc>>,
    reminder_sent_count: i32,
    last_reminder_sent: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    payment_method: Option<String>,
    payment_reference: Option<String>,
}

impl InvoiceRow {
    fn to_invoice(self) -> Invoice {
        let items: Vec<InvoiceItem> = serde_json::from_value(self.items).unwrap_or_default();

        let status = match self.status.as_str() {
            "draft" => InvoiceStatus::Draft,
            "sent" => InvoiceStatus::Sent,
            "viewed" => InvoiceStatus::Viewed,
            "partial" => InvoiceStatus::Partial,
            "paid" => InvoiceStatus::Paid,
            "overdue" => InvoiceStatus::Overdue,
            "cancelled" => InvoiceStatus::Cancelled,
            _ => InvoiceStatus::Draft,
        };

        Invoice {
            id: self.id,
            user_id: self.user_id,
            client_id: self.client_id,
            invoice_number: self.invoice_number,
            status,
            issue_date: self.issue_date,
            due_date: self.due_date,
            subtotal: self.subtotal,
            tax_amount: self.tax_amount,
            discount_amount: self.discount_amount,
            total_amount: self.total_amount,
            amount_paid: self.amount_paid,
            items,
            notes: self.notes,
            terms: self.terms,
            tax_calculation: self.tax_calculation,
            tax_included: self.tax_included,
            tax_label: self.tax_label,
            tax_id: self.tax_id,
            pdf_url: self.pdf_url,
            receipt_image_url: self.receipt_image_url,
            sent_at: self.sent_at,
            paid_at: self.paid_at,
            reminder_sent_count: self.reminder_sent_count,
            last_reminder_sent: self.last_reminder_sent,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }

    fn to_invoice_detail(self) -> InvoiceDetailResponse {
        let items: Vec<InvoiceItem> = serde_json::from_value(self.items).unwrap_or_default();

        let status = match self.status.as_str() {
            "draft" => InvoiceStatus::Draft,
            "sent" => InvoiceStatus::Sent,
            "viewed" => InvoiceStatus::Viewed,
            "partial" => InvoiceStatus::Partial,
            "paid" => InvoiceStatus::Paid,
            "overdue" => InvoiceStatus::Overdue,
            "cancelled" => InvoiceStatus::Cancelled,
            _ => InvoiceStatus::Draft,
        };

        let balance_due = self.total_amount - self.amount_paid;

        InvoiceDetailResponse {
            id: self.id,
            invoice_number: self.invoice_number,
            status,
            client_id: self.client_id,
            client_name: self.client_name.unwrap_or_default(),
            client_email: self.client_email,
            client_phone: self.client_phone,
            client_address: self.client_address,
            issue_date: self.issue_date,
            due_date: self.due_date,
            subtotal: self.subtotal,
            tax_amount: self.tax_amount,
            discount_amount: self.discount_amount,
            total_amount: self.total_amount,
            amount_paid: self.amount_paid,
            balance_due,
            items,
            notes: self.notes,
            terms: self.terms,
            tax_calculation: self.tax_calculation,
            tax_included: self.tax_included,
            tax_label: self.tax_label,
            tax_id: self.tax_id,
            pdf_url: self.pdf_url,
            receipt_image_url: self.receipt_image_url,
            sent_at: self.sent_at,
            paid_at: self.paid_at,
            reminder_sent_count: self.reminder_sent_count,
            last_reminder_sent: self.last_reminder_sent,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

impl InvoiceInsertRow {
    fn to_invoice(self) -> Invoice {
        let items: Vec<InvoiceItem> = serde_json::from_value(self.items).unwrap_or_default();

        let status = match self.status.as_str() {
            "draft" => InvoiceStatus::Draft,
            "sent" => InvoiceStatus::Sent,
            "viewed" => InvoiceStatus::Viewed,
            "partial" => InvoiceStatus::Partial,
            "paid" => InvoiceStatus::Paid,
            "overdue" => InvoiceStatus::Overdue,
            "cancelled" => InvoiceStatus::Cancelled,
            _ => InvoiceStatus::Draft,
        };

        Invoice {
            id: self.id,
            user_id: self.user_id,
            client_id: self.client_id,
            invoice_number: self.invoice_number,
            status,
            issue_date: self.issue_date,
            due_date: self.due_date,
            subtotal: self.subtotal,
            tax_amount: self.tax_amount,
            discount_amount: self.discount_amount,
            total_amount: self.total_amount,
            amount_paid: self.amount_paid,
            items,
            notes: self.notes,
            terms: self.terms,
            tax_calculation: self.tax_calculation,
            tax_included: self.tax_included,
            tax_label: self.tax_label,
            tax_id: self.tax_id,
            pdf_url: self.pdf_url,
            receipt_image_url: self.receipt_image_url,
            sent_at: self.sent_at,
            paid_at: self.paid_at,
            reminder_sent_count: self.reminder_sent_count,
            last_reminder_sent: self.last_reminder_sent,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}
