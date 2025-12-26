use sqlx::{PgPool, QueryBuilder, Postgres, Row};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::domain::models::{Client, ClientResponse, ClientStats};

#[derive(Clone)]
pub struct ClientRepository {
    db: PgPool,
}

impl ClientRepository {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn create(
        &self,
        user_id: Uuid,
        name: String,
        email: Option<String>,
        phone: Option<String>,
        company_name: Option<String>,
        billing_address: Option<serde_json::Value>,
        payment_terms: Option<i32>,
        tax_exempt: Option<bool>,
        notes: Option<String>,
    ) -> Result<Client, sqlx::Error> {
        let client = sqlx::query_as::<_, ClientRow>(
            r#"
            INSERT INTO clients (
                id, user_id, name, email, phone, company_name,
                billing_address, payment_terms, tax_exempt, notes,
                created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(&name)
        .bind(&email)
        .bind(&phone)
        .bind(&company_name)
        .bind(&billing_address)
        .bind(payment_terms.unwrap_or(30))
        .bind(tax_exempt.unwrap_or(false))
        .bind(&notes)
        .bind(Utc::now())
        .bind(Utc::now())
        .fetch_one(&self.db)
        .await?;

        Ok(client.to_client())
    }

    pub async fn find_by_id(&self, user_id: Uuid, client_id: Uuid) -> Result<Option<Client>, sqlx::Error> {
        let client = sqlx::query_as::<_, ClientRow>(
            "SELECT * FROM clients WHERE id = $1 AND user_id = $2"
        )
        .bind(client_id)
        .bind(user_id)
        .fetch_optional(&self.db)
        .await?;

        Ok(client.map(|c| c.to_client()))
    }

    pub async fn list(
        &self,
        user_id: Uuid,
        search: Option<String>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<ClientResponse>, sqlx::Error> {
        let mut query_builder = QueryBuilder::<Postgres>::new(
            r#"
            SELECT
                c.*,
                COALESCE(SUM(i.total_amount)::float8, 0.0::float8) as total_invoiced,
                COALESCE(SUM(i.amount_paid)::float8, 0.0::float8) as total_paid,
                (COALESCE(SUM(i.total_amount)::float8, 0.0::float8) - COALESCE(SUM(i.amount_paid)::float8, 0.0::float8)) as outstanding_balance,
                0 as average_payment_days,
                MAX(i.issue_date) as last_invoice_date
            FROM clients c
            LEFT JOIN invoices i ON c.id = i.client_id
            WHERE c.user_id = "#,
        );

        query_builder.push_bind(user_id);

        if let Some(s) = search {
            query_builder.push(" AND (c.name ILIKE ");
            query_builder.push_bind(format!("%{}%", s));
            query_builder.push(" OR c.email ILIKE ");
            query_builder.push_bind(format!("%{}%", s));
            query_builder.push(")");
        }

        query_builder.push(" GROUP BY c.id ORDER BY c.created_at DESC");

        if let Some(l) = limit {
            query_builder.push(" LIMIT ");
            query_builder.push_bind(l);
        }

        if let Some(o) = offset {
            query_builder.push(" OFFSET ");
            query_builder.push_bind(o);
        }

        let clients = query_builder
            .build_query_as()
            .fetch_all(&self.db)
            .await?;

        Ok(clients)
    }

    pub async fn update(
        &self,
        user_id: Uuid,
        client_id: Uuid,
        name: Option<String>,
        email: Option<String>,
        phone: Option<String>,
        company_name: Option<String>,
        billing_address: Option<serde_json::Value>,
        payment_terms: Option<i32>,
        tax_exempt: Option<bool>,
        notes: Option<String>,
    ) -> Result<Client, sqlx::Error> {
        let mut query_builder = QueryBuilder::<Postgres>::new(
            "UPDATE clients SET updated_at = "
        );
        query_builder.push_bind(Utc::now());

        if let Some(ref name) = name {
            query_builder.push(", name = ");
            query_builder.push_bind(name);
        }

        if let Some(ref email) = email {
            query_builder.push(", email = ");
            query_builder.push_bind(email);
        }

        if let Some(ref phone) = phone {
            query_builder.push(", phone = ");
            query_builder.push_bind(phone);
        }

        if let Some(ref company_name) = company_name {
            query_builder.push(", company_name = ");
            query_builder.push_bind(company_name);
        }

        if let Some(ref billing_address) = billing_address {
            query_builder.push(", billing_address = ");
            query_builder.push_bind(billing_address);
        }

        if let Some(payment_terms) = payment_terms {
            query_builder.push(", payment_terms = ");
            query_builder.push_bind(payment_terms);
        }

        if let Some(tax_exempt) = tax_exempt {
            query_builder.push(", tax_exempt = ");
            query_builder.push_bind(tax_exempt);
        }

        if let Some(ref notes) = notes {
            query_builder.push(", notes = ");
            query_builder.push_bind(notes);
        }

        query_builder.push(" WHERE id = ");
        query_builder.push_bind(client_id);
        query_builder.push(" AND user_id = ");
        query_builder.push_bind(user_id);
        query_builder.push(" RETURNING *");

        let client = query_builder
            .build_query_as::<ClientRow>()
            .fetch_one(&self.db)
            .await?;

        Ok(client.to_client())
    }

    pub async fn delete(&self, user_id: Uuid, client_id: Uuid) -> Result<(), sqlx::Error> {
        let result = sqlx::query(
            "DELETE FROM clients WHERE id = $1 AND user_id = $2"
        )
        .bind(client_id)
        .bind(user_id)
        .execute(&self.db)
        .await?;

        if result.rows_affected() == 0 {
            return Err(sqlx::Error::RowNotFound);
        }

        Ok(())
    }

    pub async fn get_stats(&self, user_id: Uuid) -> Result<ClientStats, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT
                COUNT(DISTINCT c.id) as total_clients,
                COUNT(DISTINCT CASE WHEN i.status != 'cancelled' THEN c.id END) as active_clients,
                COALESCE(SUM(i.total_amount)::float8, 0.0::float8) as total_invoiced,
                COALESCE(SUM(i.amount_paid)::float8, 0.0::float8) as total_paid,
                (COALESCE(SUM(i.total_amount)::float8, 0.0::float8) - COALESCE(SUM(i.amount_paid)::float8, 0.0::float8)) as outstanding_balance,
                0.0::float8 as avg_payment_days
            FROM clients c
            LEFT JOIN invoices i ON c.id = i.client_id
            WHERE c.user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        let total_clients: i64 = row.get("total_clients");
        let active_clients: i64 = row.get("active_clients");
        let total_invoiced: f64 = row.get("total_invoiced");
        let total_paid: f64 = row.get("total_paid");
        let outstanding_balance: f64 = row.get("outstanding_balance");
        let avg_payment_days: f64 = row.get("avg_payment_days");

        Ok(ClientStats {
            total_clients,
            active_clients,
            total_invoiced,
            total_paid,
            outstanding_balance,
            avg_payment_days,
        })
    }

    pub async fn get_client_invoices(
        &self,
        user_id: Uuid,
        client_id: Uuid,
    ) -> Result<Vec<crate::domain::models::InvoiceResponse>, sqlx::Error> {
        let invoices = sqlx::query_as::<_, crate::domain::models::InvoiceResponse>(
            r#"
            SELECT
                i.id, i.invoice_number, i.status, c.name as client_name, c.email as client_email,
                i.issue_date, i.due_date, i.total_amount,
                (i.total_amount - i.amount_paid) as balance_due,
                i.created_at,
                0 as days_until_due,
                (i.due_date < CURRENT_DATE AND i.status != 'paid' AND i.status != 'cancelled') as is_overdue
            FROM invoices i
            JOIN clients c ON i.client_id = c.id
            WHERE i.user_id = $1 AND i.client_id = $2
            ORDER BY i.created_at DESC
            "#,
        )
        .bind(user_id)
        .bind(client_id)
        .fetch_all(&self.db)
        .await?;

        Ok(invoices)
    }
}

#[derive(sqlx::FromRow)]
struct ClientRow {
    id: Uuid,
    user_id: Uuid,
    name: String,
    email: Option<String>,
    phone: Option<String>,
    company_name: Option<String>,
    billing_address: Option<serde_json::Value>,
    payment_terms: i32,
    tax_exempt: bool,
    tax_exempt_certificate: Option<String>,
    notes: Option<String>,
    total_invoiced: f64,
    total_paid: f64,
    average_payment_days: Option<i32>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl ClientRow {
    fn to_client(self) -> Client {
        Client {
            id: self.id,
            user_id: self.user_id,
            name: self.name,
            email: self.email,
            phone: self.phone,
            company_name: self.company_name,
            billing_address: self.billing_address,
            payment_terms: self.payment_terms,
            tax_exempt: self.tax_exempt,
            tax_exempt_certificate: self.tax_exempt_certificate,
            notes: self.notes,
            total_invoiced: self.total_invoiced,
            total_paid: self.total_paid,
            average_payment_days: self.average_payment_days,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}
