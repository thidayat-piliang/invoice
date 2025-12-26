use async_trait::async_trait;
use uuid::Uuid;
use chrono::NaiveDate;
use serde::Serialize;

use crate::domain::models::InvoiceStatus;

#[async_trait]
pub trait ReportRepository: Send + Sync {
    /// Get dashboard overview statistics
    async fn get_overview_stats(&self, user_id: Uuid) -> Result<OverviewStats, sqlx::Error>;

    /// Get income report for date range
    async fn get_income_report(
        &self,
        user_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<IncomeReport, sqlx::Error>;

    /// Get expenses report for date range
    async fn get_expenses_report(
        &self,
        user_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<ExpensesReport, sqlx::Error>;

    /// Get tax report for date range
    async fn get_tax_report(
        &self,
        user_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<TaxReport, sqlx::Error>;

    /// Get aging report (accounts receivable aging)
    async fn get_aging_report(&self, user_id: Uuid) -> Result<AgingReport, sqlx::Error>;
}

#[derive(Debug, Clone, Serialize)]
pub struct OverviewStats {
    pub total_revenue: f64,
    pub total_outstanding: f64,
    pub paid_invoices: i64,
    pub overdue_invoices: i64,
    pub total_expenses: f64,
    pub net_profit: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct IncomeReport {
    pub total_income: f64,
    pub by_month: Vec<IncomeByMonth>,
    pub by_client: Vec<IncomeByClient>,
}

#[derive(Debug, Clone, Serialize)]
pub struct IncomeByMonth {
    pub month: String,
    pub amount: f64,
    pub invoice_count: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct IncomeByClient {
    pub client_id: Uuid,
    pub client_name: String,
    pub total_amount: f64,
    pub invoice_count: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExpensesReport {
    pub total_expenses: f64,
    pub by_category: Vec<ExpensesByCategory>,
    pub by_month: Vec<ExpensesByMonth>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExpensesByCategory {
    pub category: String,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExpensesByMonth {
    pub month: String,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct TaxReport {
    pub total_tax_collected: f64,
    pub total_tax_deductible: f64,
    pub by_state: Vec<TaxByState>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TaxByState {
    pub state_code: String,
    pub tax_amount: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgingReport {
    pub current: f64,
    pub one_to_thirty_days: f64,
    pub thirty_one_to_sixty_days: f64,
    pub sixty_one_to_ninety_days: f64,
    pub over_ninety_days: f64,
}
