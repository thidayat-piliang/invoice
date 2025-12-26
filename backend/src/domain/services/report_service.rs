use std::sync::Arc;
use uuid::Uuid;
use chrono::NaiveDate;

use crate::domain::repositories::report_repository::{
    ReportRepository, OverviewStats, IncomeReport, ExpensesReport, TaxReport, AgingReport,
};

#[derive(Clone)]
pub struct ReportService<R: ReportRepository> {
    report_repo: Arc<R>,
}

impl<R: ReportRepository> ReportService<R> {
    pub fn new(report_repo: Arc<R>) -> Self {
        Self { report_repo }
    }

    pub async fn get_overview_stats(&self, user_id: Uuid) -> Result<OverviewStats, sqlx::Error> {
        self.report_repo.get_overview_stats(user_id).await
    }

    pub async fn get_income_report(
        &self,
        user_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<IncomeReport, sqlx::Error> {
        self.report_repo.get_income_report(user_id, start_date, end_date).await
    }

    pub async fn get_expenses_report(
        &self,
        user_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<ExpensesReport, sqlx::Error> {
        self.report_repo.get_expenses_report(user_id, start_date, end_date).await
    }

    pub async fn get_tax_report(
        &self,
        user_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<TaxReport, sqlx::Error> {
        self.report_repo.get_tax_report(user_id, start_date, end_date).await
    }

    pub async fn get_aging_report(&self, user_id: Uuid) -> Result<AgingReport, sqlx::Error> {
        self.report_repo.get_aging_report(user_id).await
    }

    pub async fn export_report(
        &self,
        user_id: Uuid,
        report_type: &str,
        format: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<String, sqlx::Error> {
        // Generate export file path
        let timestamp = chrono::Utc::now().timestamp();
        let filename = format!("{}_{}_{}_{}.{}", report_type, start_date, end_date, timestamp, format);

        // In a real implementation, this would:
        // 1. Generate the actual file (PDF/CSV)
        // 2. Upload to S3 or local storage
        // 3. Return a signed URL

        // For now, return a mock URL path
        Ok(format!("/api/v1/reports/export/{}", filename))
    }
}
