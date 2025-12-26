use sqlx::{PgPool, QueryBuilder, Postgres, Row};
use uuid::Uuid;
use chrono::{DateTime, NaiveDate, Utc};

use crate::domain::models::{Expense, ExpenseResponse, ExpenseStats, ExpenseCategory};

#[derive(Clone)]
pub struct ExpenseRepository {
    db: PgPool,
}

impl ExpenseRepository {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn create(
        &self,
        user_id: Uuid,
        amount: f64,
        currency: String,
        category: ExpenseCategory,
        vendor: Option<String>,
        description: Option<String>,
        receipt_image_url: Option<String>,
        date_incurred: NaiveDate,
        tax_deductible: Option<bool>,
    ) -> Result<Expense, sqlx::Error> {
        let expense = sqlx::query_as::<_, ExpenseRow>(
            r#"
            INSERT INTO expenses (
                id, user_id, amount, currency, category, vendor,
                description, receipt_image_url, date_incurred, tax_deductible,
                created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(amount)
        .bind(currency)
        .bind(category.to_string())
        .bind(&vendor)
        .bind(description)
        .bind(&receipt_image_url)
        .bind(date_incurred)
        .bind(tax_deductible.unwrap_or(true))
        .bind(Utc::now())
        .bind(Utc::now())
        .fetch_one(&self.db)
        .await?;

        Ok(expense.to_expense())
    }

    pub async fn find_by_id(&self, user_id: Uuid, expense_id: Uuid) -> Result<Option<Expense>, sqlx::Error> {
        let expense = sqlx::query_as::<_, ExpenseRow>(
            "SELECT * FROM expenses WHERE id = $1 AND user_id = $2"
        )
        .bind(expense_id)
        .bind(user_id)
        .fetch_optional(&self.db)
        .await?;

        Ok(expense.map(|e| e.to_expense()))
    }

    pub async fn list(
        &self,
        user_id: Uuid,
        category: Option<ExpenseCategory>,
        date_from: Option<NaiveDate>,
        date_to: Option<NaiveDate>,
        tax_deductible: Option<bool>,
        search: Option<String>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<ExpenseResponse>, sqlx::Error> {
        let mut query_builder = QueryBuilder::<Postgres>::new(
            r#"
            SELECT * FROM expenses
            WHERE user_id = "#
        );

        query_builder.push_bind(user_id);

        if let Some(c) = category {
            query_builder.push(" AND category = ");
            query_builder.push_bind(c.to_string());
        }

        if let Some(df) = date_from {
            query_builder.push(" AND date_incurred >= ");
            query_builder.push_bind(df);
        }

        if let Some(dt) = date_to {
            query_builder.push(" AND date_incurred <= ");
            query_builder.push_bind(dt);
        }

        if let Some(td) = tax_deductible {
            query_builder.push(" AND tax_deductible = ");
            query_builder.push_bind(td);
        }

        if let Some(s) = search {
            query_builder.push(" AND (vendor ILIKE ");
            query_builder.push_bind(format!("%{}%", s));
            query_builder.push(" OR description ILIKE ");
            query_builder.push_bind(format!("%{}%", s));
            query_builder.push(")");
        }

        query_builder.push(" ORDER BY created_at DESC");

        if let Some(l) = limit {
            query_builder.push(" LIMIT ");
            query_builder.push_bind(l);
        }

        if let Some(o) = offset {
            query_builder.push(" OFFSET ");
            query_builder.push_bind(o);
        }

        let expenses = query_builder
            .build_query_as::<ExpenseRow>()
            .fetch_all(&self.db)
            .await?;

        Ok(expenses.into_iter().map(|e: ExpenseRow| e.to_expense_response()).collect())
    }

    pub async fn update(
        &self,
        user_id: Uuid,
        expense_id: Uuid,
        amount: Option<f64>,
        category: Option<ExpenseCategory>,
        vendor: Option<String>,
        description: Option<String>,
        receipt_image_url: Option<String>,
        date_incurred: Option<NaiveDate>,
        tax_deductible: Option<bool>,
    ) -> Result<Expense, sqlx::Error> {
        let mut query_builder = QueryBuilder::<Postgres>::new(
            "UPDATE expenses SET updated_at = "
        );
        query_builder.push_bind(Utc::now());

        if let Some(a) = amount {
            query_builder.push(", amount = ");
            query_builder.push_bind(a);
        }

        if let Some(c) = category {
            query_builder.push(", category = ");
            query_builder.push_bind(c.to_string());
        }

        if let Some(ref v) = vendor {
            query_builder.push(", vendor = ");
            query_builder.push_bind(v);
        }

        if let Some(ref d) = description {
            query_builder.push(", description = ");
            query_builder.push_bind(d);
        }

        if let Some(ref r) = receipt_image_url {
            query_builder.push(", receipt_image_url = ");
            query_builder.push_bind(r);
        }

        if let Some(di) = date_incurred {
            query_builder.push(", date_incurred = ");
            query_builder.push_bind(di);
        }

        if let Some(td) = tax_deductible {
            query_builder.push(", tax_deductible = ");
            query_builder.push_bind(td);
        }

        query_builder.push(" WHERE id = ");
        query_builder.push_bind(expense_id);
        query_builder.push(" AND user_id = ");
        query_builder.push_bind(user_id);
        query_builder.push(" RETURNING *");

        let expense = query_builder
            .build_query_as::<ExpenseRow>()
            .fetch_one(&self.db)
            .await?;

        Ok(expense.to_expense())
    }

    pub async fn delete(&self, user_id: Uuid, expense_id: Uuid) -> Result<(), sqlx::Error> {
        let result = sqlx::query(
            "DELETE FROM expenses WHERE id = $1 AND user_id = $2"
        )
        .bind(expense_id)
        .bind(user_id)
        .execute(&self.db)
        .await?;

        if result.rows_affected() == 0 {
            return Err(sqlx::Error::RowNotFound);
        }

        Ok(())
    }

    pub async fn get_stats(&self, user_id: Uuid) -> Result<ExpenseStats, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT
                COALESCE(SUM(amount)::float8, 0.0::float8) as total_expenses,
                COALESCE(SUM(CASE WHEN tax_deductible THEN amount END)::float8, 0.0::float8) as tax_deductible,
                COALESCE(AVG(amount)::float8, 0.0::float8) as monthly_average,
                COALESCE(SUM(CASE WHEN category = 'supplies' THEN amount END)::float8, 0.0::float8) as supplies,
                COALESCE(SUM(CASE WHEN category = 'office_supplies' THEN amount END)::float8, 0.0::float8) as office_supplies,
                COALESCE(SUM(CASE WHEN category = 'travel' THEN amount END)::float8, 0.0::float8) as travel,
                COALESCE(SUM(CASE WHEN category = 'equipment' THEN amount END)::float8, 0.0::float8) as equipment,
                COALESCE(SUM(CASE WHEN category = 'software' THEN amount END)::float8, 0.0::float8) as software,
                COALESCE(SUM(CASE WHEN category = 'marketing' THEN amount END)::float8, 0.0::float8) as marketing,
                COALESCE(SUM(CASE WHEN category = 'utilities' THEN amount END)::float8, 0.0::float8) as utilities,
                COALESCE(SUM(CASE WHEN category = 'other' THEN amount END)::float8, 0.0::float8) as other
            FROM expenses
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        let mut by_category = std::collections::HashMap::new();
        by_category.insert("supplies".to_string(), row.get::<f64, _>("supplies"));
        by_category.insert("office_supplies".to_string(), row.get::<f64, _>("office_supplies"));
        by_category.insert("travel".to_string(), row.get::<f64, _>("travel"));
        by_category.insert("equipment".to_string(), row.get::<f64, _>("equipment"));
        by_category.insert("software".to_string(), row.get::<f64, _>("software"));
        by_category.insert("marketing".to_string(), row.get::<f64, _>("marketing"));
        by_category.insert("utilities".to_string(), row.get::<f64, _>("utilities"));
        by_category.insert("other".to_string(), row.get::<f64, _>("other"));

        Ok(ExpenseStats {
            total_expenses: row.get("total_expenses"),
            tax_deductible: row.get("tax_deductible"),
            by_category,
            monthly_average: row.get("monthly_average"),
        })
    }
}

#[derive(sqlx::FromRow)]
struct ExpenseRow {
    id: Uuid,
    user_id: Uuid,
    amount: f64,
    currency: String,
    category: String,
    vendor: Option<String>,
    description: Option<String>,
    receipt_image_url: Option<String>,
    date_incurred: NaiveDate,
    tax_deductible: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl ExpenseRow {
    fn to_expense(self) -> Expense {
        let category = match self.category.as_str() {
            "supplies" => ExpenseCategory::Supplies,
            "office_supplies" => ExpenseCategory::OfficeSupplies,
            "travel" => ExpenseCategory::Travel,
            "equipment" => ExpenseCategory::Equipment,
            "software" => ExpenseCategory::Software,
            "marketing" => ExpenseCategory::Marketing,
            "utilities" => ExpenseCategory::Utilities,
            "other" => ExpenseCategory::Other,
            _ => ExpenseCategory::Other,
        };

        Expense {
            id: self.id,
            user_id: self.user_id,
            amount: self.amount,
            currency: self.currency,
            category,
            vendor: self.vendor,
            description: self.description,
            receipt_image_url: self.receipt_image_url,
            date_incurred: self.date_incurred,
            tax_deductible: self.tax_deductible,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }

    fn to_expense_response(self) -> ExpenseResponse {
        let category = match self.category.as_str() {
            "supplies" => ExpenseCategory::Supplies,
            "office_supplies" => ExpenseCategory::OfficeSupplies,
            "travel" => ExpenseCategory::Travel,
            "equipment" => ExpenseCategory::Equipment,
            "software" => ExpenseCategory::Software,
            "marketing" => ExpenseCategory::Marketing,
            "utilities" => ExpenseCategory::Utilities,
            "other" => ExpenseCategory::Other,
            _ => ExpenseCategory::Other,
        };

        ExpenseResponse {
            id: self.id,
            amount: self.amount,
            currency: self.currency,
            category,
            vendor: self.vendor,
            description: self.description,
            date_incurred: self.date_incurred,
            tax_deductible: self.tax_deductible,
            created_at: self.created_at,
        }
    }
}
