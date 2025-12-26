use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;
use chrono::NaiveDate;

use crate::domain::repositories::report_repository::{
    ReportRepository, OverviewStats, IncomeReport, ExpensesReport, TaxReport, AgingReport,
    IncomeByMonth, IncomeByClient, ExpensesByCategory, ExpensesByMonth, TaxByState,
};

#[derive(Clone)]
pub struct ReportRepositoryImpl {
    db: PgPool,
}

impl ReportRepositoryImpl {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }
}

#[async_trait]
impl ReportRepository for ReportRepositoryImpl {
    async fn get_overview_stats(&self, user_id: Uuid) -> Result<OverviewStats, sqlx::Error> {
        // Total revenue (paid invoices)
        let total_revenue: f64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(total_amount), 0.0) FROM invoices WHERE user_id = $1 AND status = 'paid'"
        )
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        // Total outstanding (sent/partial/overdue invoices)
        let total_outstanding: f64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(total_amount - amount_paid), 0.0) FROM invoices WHERE user_id = $1 AND status IN ('sent', 'partial', 'overdue')"
        )
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        // Paid invoices count
        let paid_invoices: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM invoices WHERE user_id = $1 AND status = 'paid'"
        )
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        // Overdue invoices count
        let overdue_invoices: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM invoices WHERE user_id = $1 AND status = 'overdue'"
        )
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        // Total expenses (mock - would need expenses table)
        let total_expenses: f64 = 0.0;

        // Net profit
        let net_profit = total_revenue - total_expenses;

        Ok(OverviewStats {
            total_revenue,
            total_outstanding,
            paid_invoices,
            overdue_invoices,
            total_expenses,
            net_profit,
        })
    }

    async fn get_income_report(
        &self,
        user_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<IncomeReport, sqlx::Error> {
        // Total income
        let total_income: f64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(total_amount), 0.0) FROM invoices WHERE user_id = $1 AND status = 'paid' AND issue_date BETWEEN $2 AND $3"
        )
        .bind(user_id)
        .bind(start_date)
        .bind(end_date)
        .fetch_one(&self.db)
        .await?;

        // By month
        let by_month_rows = sqlx::query(
            r#"
            SELECT
                TO_CHAR(issue_date, 'YYYY-MM') as month,
                SUM(total_amount) as amount,
                COUNT(*) as invoice_count
            FROM invoices
            WHERE user_id = $1 AND status = 'paid' AND issue_date BETWEEN $2 AND $3
            GROUP BY TO_CHAR(issue_date, 'YYYY-MM')
            ORDER BY month
            "#
        )
        .bind(user_id)
        .bind(start_date)
        .bind(end_date)
        .fetch_all(&self.db)
        .await?;

        let by_month: Vec<IncomeByMonth> = by_month_rows
            .iter()
            .map(|row| IncomeByMonth {
                month: row.get("month"),
                amount: row.get("amount"),
                invoice_count: row.get("invoice_count"),
            })
            .collect();

        // By client
        let by_client_rows = sqlx::query(
            r#"
            SELECT
                c.id as client_id,
                c.name as client_name,
                SUM(i.total_amount) as total_amount,
                COUNT(*) as invoice_count
            FROM invoices i
            JOIN clients c ON i.client_id = c.id
            WHERE i.user_id = $1 AND i.status = 'paid' AND i.issue_date BETWEEN $2 AND $3
            GROUP BY c.id, c.name
            ORDER BY total_amount DESC
            "#
        )
        .bind(user_id)
        .bind(start_date)
        .bind(end_date)
        .fetch_all(&self.db)
        .await?;

        let by_client: Vec<IncomeByClient> = by_client_rows
            .iter()
            .map(|row| IncomeByClient {
                client_id: row.get("client_id"),
                client_name: row.get("client_name"),
                total_amount: row.get("total_amount"),
                invoice_count: row.get("invoice_count"),
            })
            .collect();

        Ok(IncomeReport {
            total_income,
            by_month,
            by_client,
        })
    }

    async fn get_expenses_report(
        &self,
        _user_id: Uuid,
        _start_date: NaiveDate,
        _end_date: NaiveDate,
    ) -> Result<ExpensesReport, sqlx::Error> {
        // Note: This would require an expenses table
        // For now, return empty structure
        Ok(ExpensesReport {
            total_expenses: 0.0,
            by_category: vec![],
            by_month: vec![],
        })
    }

    async fn get_tax_report(
        &self,
        user_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<TaxReport, sqlx::Error> {
        // Total tax collected
        let total_tax_collected: f64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(tax_amount), 0.0) FROM invoices WHERE user_id = $1 AND status = 'paid' AND issue_date BETWEEN $2 AND $3"
        )
        .bind(user_id)
        .bind(start_date)
        .bind(end_date)
        .fetch_one(&self.db)
        .await?;

        // Tax deductible (mock - would need expense tax tracking)
        let total_tax_deductible: f64 = 0.0;

        // By state (from client addresses)
        let by_state_rows = sqlx::query(
            r#"
            SELECT
                COALESCE(c.billing_address->>'state', 'Unknown') as state_code,
                SUM(i.tax_amount) as tax_amount
            FROM invoices i
            JOIN clients c ON i.client_id = c.id
            WHERE i.user_id = $1 AND i.status = 'paid' AND i.issue_date BETWEEN $2 AND $3
            GROUP BY COALESCE(c.billing_address->>'state', 'Unknown')
            ORDER BY tax_amount DESC
            "#
        )
        .bind(user_id)
        .bind(start_date)
        .bind(end_date)
        .fetch_all(&self.db)
        .await?;

        let by_state: Vec<TaxByState> = by_state_rows
            .iter()
            .map(|row| TaxByState {
                state_code: row.get("state_code"),
                tax_amount: row.get("tax_amount"),
            })
            .collect();

        Ok(TaxReport {
            total_tax_collected,
            total_tax_deductible,
            by_state,
        })
    }

    async fn get_aging_report(&self, user_id: Uuid) -> Result<AgingReport, sqlx::Error> {
        let today = chrono::Utc::now().naive_utc().date();

        // Current (not yet due)
        let current: f64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(total_amount - amount_paid), 0.0) FROM invoices WHERE user_id = $1 AND status IN ('sent', 'partial') AND due_date >= $2"
        )
        .bind(user_id)
        .bind(today)
        .fetch_one(&self.db)
        .await?;

        // 1-30 days overdue
        let one_to_thirty_days: f64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(total_amount - amount_paid), 0.0) FROM invoices WHERE user_id = $1 AND status IN ('overdue', 'partial') AND due_date < $2 AND due_date >= $3"
        )
        .bind(user_id)
        .bind(today)
        .bind(today - chrono::Duration::days(30))
        .fetch_one(&self.db)
        .await?;

        // 31-60 days overdue
        let thirty_one_to_sixty_days: f64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(total_amount - amount_paid), 0.0) FROM invoices WHERE user_id = $1 AND status IN ('overdue', 'partial') AND due_date < $2 AND due_date >= $3"
        )
        .bind(user_id)
        .bind(today - chrono::Duration::days(30))
        .bind(today - chrono::Duration::days(60))
        .fetch_one(&self.db)
        .await?;

        // 61-90 days overdue
        let sixty_one_to_ninety_days: f64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(total_amount - amount_paid), 0.0) FROM invoices WHERE user_id = $1 AND status IN ('overdue', 'partial') AND due_date < $2 AND due_date >= $3"
        )
        .bind(user_id)
        .bind(today - chrono::Duration::days(60))
        .bind(today - chrono::Duration::days(90))
        .fetch_one(&self.db)
        .await?;

        // Over 90 days overdue
        let over_ninety_days: f64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(total_amount - amount_paid), 0.0) FROM invoices WHERE user_id = $1 AND status IN ('overdue', 'partial') AND due_date < $2"
        )
        .bind(user_id)
        .bind(today - chrono::Duration::days(90))
        .fetch_one(&self.db)
        .await?;

        Ok(AgingReport {
            current,
            one_to_thirty_days,
            thirty_one_to_sixty_days,
            sixty_one_to_ninety_days,
            over_ninety_days,
        })
    }
}
