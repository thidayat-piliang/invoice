use std::sync::Arc;
use uuid::Uuid;
use chrono::NaiveDate;
use thiserror::Error;

use crate::domain::services::ReportService;
use crate::domain::repositories::report_repository::{
    OverviewStats, IncomeReport, ExpensesReport, TaxReport, AgingReport,
};

#[derive(Debug, Error)]
pub enum ReportError {
    #[error("Database error: {0}")]
    DatabaseError(String),
}

impl From<sqlx::Error> for ReportError {
    fn from(err: sqlx::Error) -> Self {
        ReportError::DatabaseError(err.to_string())
    }
}

// GetOverviewStatsUseCase
#[derive(Clone)]
pub struct GetOverviewStatsUseCase {
    report_service: Arc<ReportService<crate::infrastructure::repositories::ReportRepositoryImpl>>,
}

impl GetOverviewStatsUseCase {
    pub fn new(report_service: Arc<ReportService<crate::infrastructure::repositories::ReportRepositoryImpl>>) -> Self {
        Self { report_service }
    }

    pub async fn execute(&self, user_id: Uuid) -> Result<OverviewStats, ReportError> {
        Ok(self.report_service.get_overview_stats(user_id).await?)
    }
}

// GetIncomeReportUseCase
#[derive(Clone)]
pub struct GetIncomeReportUseCase {
    report_service: Arc<ReportService<crate::infrastructure::repositories::ReportRepositoryImpl>>,
}

impl GetIncomeReportUseCase {
    pub fn new(report_service: Arc<ReportService<crate::infrastructure::repositories::ReportRepositoryImpl>>) -> Self {
        Self { report_service }
    }

    pub async fn execute(
        &self,
        user_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<IncomeReport, ReportError> {
        Ok(self.report_service.get_income_report(user_id, start_date, end_date).await?)
    }
}

// GetExpensesReportUseCase
#[derive(Clone)]
pub struct GetExpensesReportUseCase {
    report_service: Arc<ReportService<crate::infrastructure::repositories::ReportRepositoryImpl>>,
}

impl GetExpensesReportUseCase {
    pub fn new(report_service: Arc<ReportService<crate::infrastructure::repositories::ReportRepositoryImpl>>) -> Self {
        Self { report_service }
    }

    pub async fn execute(
        &self,
        user_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<ExpensesReport, ReportError> {
        Ok(self.report_service.get_expenses_report(user_id, start_date, end_date).await?)
    }
}

// GetTaxReportUseCase
#[derive(Clone)]
pub struct GetTaxReportUseCase {
    report_service: Arc<ReportService<crate::infrastructure::repositories::ReportRepositoryImpl>>,
}

impl GetTaxReportUseCase {
    pub fn new(report_service: Arc<ReportService<crate::infrastructure::repositories::ReportRepositoryImpl>>) -> Self {
        Self { report_service }
    }

    pub async fn execute(
        &self,
        user_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<TaxReport, ReportError> {
        Ok(self.report_service.get_tax_report(user_id, start_date, end_date).await?)
    }
}

// GetAgingReportUseCase
#[derive(Clone)]
pub struct GetAgingReportUseCase {
    report_service: Arc<ReportService<crate::infrastructure::repositories::ReportRepositoryImpl>>,
}

impl GetAgingReportUseCase {
    pub fn new(report_service: Arc<ReportService<crate::infrastructure::repositories::ReportRepositoryImpl>>) -> Self {
        Self { report_service }
    }

    pub async fn execute(&self, user_id: Uuid) -> Result<AgingReport, ReportError> {
        Ok(self.report_service.get_aging_report(user_id).await?)
    }
}

// ExportReportUseCase
#[derive(Clone)]
pub struct ExportReportUseCase {
    report_service: Arc<ReportService<crate::infrastructure::repositories::ReportRepositoryImpl>>,
}

impl ExportReportUseCase {
    pub fn new(report_service: Arc<ReportService<crate::infrastructure::repositories::ReportRepositoryImpl>>) -> Self {
        Self { report_service }
    }

    pub async fn execute(
        &self,
        user_id: Uuid,
        report_type: String,
        format: String,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<String, ReportError> {
        Ok(self.report_service.export_report(user_id, &report_type, &format, start_date, end_date).await?)
    }
}
