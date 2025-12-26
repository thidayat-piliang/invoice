use std::sync::Arc;
use uuid::Uuid;
use chrono::NaiveDate;
use csv::Writer;

use crate::domain::repositories::report_repository::{
    ReportRepository, OverviewStats, IncomeReport, ExpensesReport, TaxReport, AgingReport,
};
use crate::domain::services::{PdfService, InvoiceItemPdf, RedisService};

#[derive(Clone)]
pub struct ReportService<R: ReportRepository> {
    report_repo: Arc<R>,
    pdf_service: Arc<PdfService>,
    redis: Option<Arc<RedisService>>,
}

impl<R: ReportRepository> ReportService<R> {
    pub fn new(report_repo: Arc<R>) -> Self {
        Self {
            report_repo,
            pdf_service: Arc::new(PdfService::new()),
            redis: None,
        }
    }

    pub fn with_redis(report_repo: Arc<R>, redis: Arc<RedisService>) -> Self {
        Self {
            report_repo,
            pdf_service: Arc::new(PdfService::new()),
            redis: Some(redis),
        }
    }

    // Helper to get cache key
    fn get_cache_key(&self, prefix: &str, user_id: Uuid, start_date: NaiveDate, end_date: NaiveDate) -> String {
        format!("report:{}:{}:{}:{}", prefix, user_id, start_date, end_date)
    }

    // Helper to get from cache
    async fn get_cached<T: serde::de::DeserializeOwned + Clone>(
        &self,
        key: &str,
    ) -> Option<T> {
        if let Some(redis) = &self.redis {
            if let Ok(Some(data)) = redis.get::<T>(key).await {
                return Some(data);
            }
        }
        None
    }

    // Helper to set cache
    async fn set_cache<T: serde::Serialize + Clone>(
        &self,
        key: &str,
        data: &T,
        ttl: u64,
    ) {
        if let Some(redis) = &self.redis {
            let _ = redis.set_with_expiration(key, data, ttl).await;
        }
    }

    pub async fn get_overview_stats(&self, user_id: Uuid) -> Result<OverviewStats, sqlx::Error> {
        // Try cache first
        let cache_key = format!("overview_stats:{}", user_id);
        if let Some(cached) = self.get_cached::<OverviewStats>(&cache_key).await {
            return Ok(cached);
        }

        let result = self.report_repo.get_overview_stats(user_id).await?;

        // Cache for 5 minutes
        self.set_cache(&cache_key, &result, 300).await;

        Ok(result)
    }

    pub async fn get_income_report(
        &self,
        user_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<IncomeReport, sqlx::Error> {
        let cache_key = self.get_cache_key("income", user_id, start_date, end_date);
        if let Some(cached) = self.get_cached::<IncomeReport>(&cache_key).await {
            return Ok(cached);
        }

        let result = self.report_repo.get_income_report(user_id, start_date, end_date).await?;
        self.set_cache(&cache_key, &result, 300).await;

        Ok(result)
    }

    pub async fn get_expenses_report(
        &self,
        user_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<ExpensesReport, sqlx::Error> {
        let cache_key = self.get_cache_key("expenses", user_id, start_date, end_date);
        if let Some(cached) = self.get_cached::<ExpensesReport>(&cache_key).await {
            return Ok(cached);
        }

        let result = self.report_repo.get_expenses_report(user_id, start_date, end_date).await?;
        self.set_cache(&cache_key, &result, 300).await;

        Ok(result)
    }

    pub async fn get_tax_report(
        &self,
        user_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<TaxReport, sqlx::Error> {
        let cache_key = self.get_cache_key("tax", user_id, start_date, end_date);
        if let Some(cached) = self.get_cached::<TaxReport>(&cache_key).await {
            return Ok(cached);
        }

        let result = self.report_repo.get_tax_report(user_id, start_date, end_date).await?;
        self.set_cache(&cache_key, &result, 300).await;

        Ok(result)
    }

    pub async fn get_aging_report(&self, user_id: Uuid) -> Result<AgingReport, sqlx::Error> {
        let cache_key = format!("aging:{}", user_id);
        if let Some(cached) = self.get_cached::<AgingReport>(&cache_key).await {
            return Ok(cached);
        }

        let result = self.report_repo.get_aging_report(user_id).await?;
        self.set_cache(&cache_key, &result, 300).await;

        Ok(result)
    }

    pub async fn export_report(
        &self,
        user_id: Uuid,
        report_type: &str,
        format: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        // Get the appropriate report data based on report_type
        match report_type {
            "income" => {
                let report = self.get_income_report(user_id, start_date, end_date).await?;
                match format {
                    "csv" => self.export_income_csv(&report),
                    "pdf" => self.export_income_pdf(&report, start_date, end_date),
                    _ => Err("Unsupported format".into()),
                }
            }
            "expenses" => {
                let report = self.get_expenses_report(user_id, start_date, end_date).await?;
                match format {
                    "csv" => self.export_expenses_csv(&report),
                    "pdf" => self.export_expenses_pdf(&report, start_date, end_date),
                    _ => Err("Unsupported format".into()),
                }
            }
            "tax" => {
                let report = self.get_tax_report(user_id, start_date, end_date).await?;
                match format {
                    "csv" => self.export_tax_csv(&report),
                    "pdf" => self.export_tax_pdf(&report, start_date, end_date),
                    _ => Err("Unsupported format".into()),
                }
            }
            "aging" => {
                let report = self.get_aging_report(user_id).await?;
                match format {
                    "csv" => self.export_aging_csv(&report),
                    "pdf" => self.export_aging_pdf(&report, start_date, end_date),
                    _ => Err("Unsupported format".into()),
                }
            }
            "overview" => {
                let report = self.get_overview_stats(user_id).await?;
                match format {
                    "csv" => self.export_overview_csv(&report),
                    "pdf" => self.export_overview_pdf(&report, start_date, end_date),
                    _ => Err("Unsupported format".into()),
                }
            }
            _ => Err("Unknown report type".into()),
        }
    }

    // CSV Export Methods
    fn export_income_csv(&self, report: &IncomeReport) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let mut wtr = Writer::from_writer(vec![]);
        wtr.write_record(["Total Income", &report.total_income.to_string()])?;
        wtr.write_record([] as [&str; 0])?;

        wtr.write_record(["By Month"])?;
        wtr.write_record(["Month", "Amount", "Invoice Count"])?;
        for item in &report.by_month {
            wtr.write_record([&item.month, &item.amount.to_string(), &item.invoice_count.to_string()])?;
        }
        wtr.write_record([] as [&str; 0])?;

        wtr.write_record(["By Client"])?;
        wtr.write_record(["Client ID", "Client Name", "Total Amount", "Invoice Count"])?;
        for item in &report.by_client {
            wtr.write_record([
                &item.client_id.to_string(),
                &item.client_name,
                &item.total_amount.to_string(),
                &item.invoice_count.to_string(),
            ])?;
        }

        Ok(wtr.into_inner()?)
    }

    fn export_expenses_csv(&self, report: &ExpensesReport) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let mut wtr = Writer::from_writer(vec![]);
        wtr.write_record(["Total Expenses", &report.total_expenses.to_string()])?;
        wtr.write_record([] as [&str; 0])?;

        wtr.write_record(["By Category"])?;
        wtr.write_record(["Category", "Amount"])?;
        for item in &report.by_category {
            wtr.write_record([&item.category, &item.amount.to_string()])?;
        }
        wtr.write_record([] as [&str; 0])?;

        wtr.write_record(["By Month"])?;
        wtr.write_record(["Month", "Amount"])?;
        for item in &report.by_month {
            wtr.write_record([&item.month, &item.amount.to_string()])?;
        }

        Ok(wtr.into_inner()?)
    }

    fn export_tax_csv(&self, report: &TaxReport) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let mut wtr = Writer::from_writer(vec![]);
        wtr.write_record(["Total Tax Collected", &report.total_tax_collected.to_string()])?;
        wtr.write_record(["Total Tax Deductible", &report.total_tax_deductible.to_string()])?;
        wtr.write_record([] as [&str; 0])?;

        wtr.write_record(["By State"])?;
        wtr.write_record(["State Code", "Tax Amount"])?;
        for item in &report.by_state {
            wtr.write_record([&item.state_code, &item.tax_amount.to_string()])?;
        }

        Ok(wtr.into_inner()?)
    }

    fn export_aging_csv(&self, report: &AgingReport) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let mut wtr = Writer::from_writer(vec![]);
        wtr.write_record(["Current", &report.current.to_string()])?;
        wtr.write_record(["1-30 Days", &report.one_to_thirty_days.to_string()])?;
        wtr.write_record(["31-60 Days", &report.thirty_one_to_sixty_days.to_string()])?;
        wtr.write_record(["61-90 Days", &report.sixty_one_to_ninety_days.to_string()])?;
        wtr.write_record(["Over 90 Days", &report.over_ninety_days.to_string()])?;

        Ok(wtr.into_inner()?)
    }

    fn export_overview_csv(&self, report: &OverviewStats) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let mut wtr = Writer::from_writer(vec![]);
        wtr.write_record(["Metric", "Value"])?;
        wtr.write_record(["Total Revenue", &report.total_revenue.to_string()])?;
        wtr.write_record(["Total Outstanding", &report.total_outstanding.to_string()])?;
        wtr.write_record(["Paid Invoices", &report.paid_invoices.to_string()])?;
        wtr.write_record(["Overdue Invoices", &report.overdue_invoices.to_string()])?;
        wtr.write_record(["Total Expenses", &report.total_expenses.to_string()])?;
        wtr.write_record(["Net Profit", &report.net_profit.to_string()])?;

        Ok(wtr.into_inner()?)
    }

    // PDF Export Methods
    fn export_income_pdf(
        &self,
        report: &IncomeReport,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let mut items = vec![
            InvoiceItemPdf {
                description: format!("Total Income: ${:.2}", report.total_income),
                quantity: 1.0,
                unit_price: report.total_income,
                total: report.total_income,
            },
        ];

        for item in &report.by_month {
            items.push(InvoiceItemPdf {
                description: format!("Month: {}", item.month),
                quantity: item.invoice_count as f64,
                unit_price: item.amount,
                total: item.amount,
            });
        }

        for item in &report.by_client {
            items.push(InvoiceItemPdf {
                description: format!("Client: {}", item.client_name),
                quantity: item.invoice_count as f64,
                unit_price: item.total_amount,
                total: item.total_amount,
            });
        }

        let pdf = self.pdf_service.generate_invoice_pdf(
            &format!("INCOME-REPORT-{}-{}", start_date, end_date),
            Some("FlashBill"),
            Some("Income Report"),
            "Report Generated",
            None,
            None,
            &start_date.to_string(),
            &end_date.to_string(),
            &items,
            report.total_income,
            0.0,
            0.0,
            report.total_income,
            Some(&format!("Income report from {} to {}", start_date, end_date)),
            None,
            None,
        )?;

        Ok(pdf)
    }

    fn export_expenses_pdf(
        &self,
        report: &ExpensesReport,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let mut items = vec![
            InvoiceItemPdf {
                description: format!("Total Expenses: ${:.2}", report.total_expenses),
                quantity: 1.0,
                unit_price: report.total_expenses,
                total: report.total_expenses,
            },
        ];

        for item in &report.by_category {
            items.push(InvoiceItemPdf {
                description: format!("Category: {}", item.category),
                quantity: 1.0,
                unit_price: item.amount,
                total: item.amount,
            });
        }

        for item in &report.by_month {
            items.push(InvoiceItemPdf {
                description: format!("Month: {}", item.month),
                quantity: 1.0,
                unit_price: item.amount,
                total: item.amount,
            });
        }

        let pdf = self.pdf_service.generate_invoice_pdf(
            &format!("EXPENSES-REPORT-{}-{}", start_date, end_date),
            Some("FlashBill"),
            Some("Expenses Report"),
            "Report Generated",
            None,
            None,
            &start_date.to_string(),
            &end_date.to_string(),
            &items,
            report.total_expenses,
            0.0,
            0.0,
            report.total_expenses,
            Some(&format!("Expenses report from {} to {}", start_date, end_date)),
            None,
            None,
        )?;

        Ok(pdf)
    }

    fn export_tax_pdf(
        &self,
        report: &TaxReport,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let mut items = vec![
            InvoiceItemPdf {
                description: format!("Total Tax Collected: ${:.2}", report.total_tax_collected),
                quantity: 1.0,
                unit_price: report.total_tax_collected,
                total: report.total_tax_collected,
            },
            InvoiceItemPdf {
                description: format!("Total Tax Deductible: ${:.2}", report.total_tax_deductible),
                quantity: 1.0,
                unit_price: report.total_tax_deductible,
                total: report.total_tax_deductible,
            },
        ];

        for item in &report.by_state {
            items.push(InvoiceItemPdf {
                description: format!("State: {}", item.state_code),
                quantity: 1.0,
                unit_price: item.tax_amount,
                total: item.tax_amount,
            });
        }

        let pdf = self.pdf_service.generate_invoice_pdf(
            &format!("TAX-REPORT-{}-{}", start_date, end_date),
            Some("FlashBill"),
            Some("Tax Report"),
            "Report Generated",
            None,
            None,
            &start_date.to_string(),
            &end_date.to_string(),
            &items,
            report.total_tax_collected,
            0.0,
            0.0,
            report.total_tax_collected,
            Some(&format!("Tax report from {} to {}", start_date, end_date)),
            None,
            None,
        )?;

        Ok(pdf)
    }

    fn export_aging_pdf(
        &self,
        report: &AgingReport,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let items = vec![
            InvoiceItemPdf {
                description: "Current (0 days)".to_string(),
                quantity: 1.0,
                unit_price: report.current,
                total: report.current,
            },
            InvoiceItemPdf {
                description: "1-30 Days".to_string(),
                quantity: 1.0,
                unit_price: report.one_to_thirty_days,
                total: report.one_to_thirty_days,
            },
            InvoiceItemPdf {
                description: "31-60 Days".to_string(),
                quantity: 1.0,
                unit_price: report.thirty_one_to_sixty_days,
                total: report.thirty_one_to_sixty_days,
            },
            InvoiceItemPdf {
                description: "61-90 Days".to_string(),
                quantity: 1.0,
                unit_price: report.sixty_one_to_ninety_days,
                total: report.sixty_one_to_ninety_days,
            },
            InvoiceItemPdf {
                description: "Over 90 Days".to_string(),
                quantity: 1.0,
                unit_price: report.over_ninety_days,
                total: report.over_ninety_days,
            },
        ];

        let total = report.current + report.one_to_thirty_days + report.thirty_one_to_sixty_days
            + report.sixty_one_to_ninety_days + report.over_ninety_days;

        let pdf = self.pdf_service.generate_invoice_pdf(
            &format!("AGING-REPORT-{}-{}", start_date, end_date),
            Some("FlashBill"),
            Some("Aging Report"),
            "Report Generated",
            None,
            None,
            &start_date.to_string(),
            &end_date.to_string(),
            &items,
            total,
            0.0,
            0.0,
            total,
            Some(&format!("Aging report from {} to {}", start_date, end_date)),
            None,
            None,
        )?;

        Ok(pdf)
    }

    fn export_overview_pdf(
        &self,
        report: &OverviewStats,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let items = vec![
            InvoiceItemPdf {
                description: "Total Revenue".to_string(),
                quantity: 1.0,
                unit_price: report.total_revenue,
                total: report.total_revenue,
            },
            InvoiceItemPdf {
                description: "Total Outstanding".to_string(),
                quantity: 1.0,
                unit_price: report.total_outstanding,
                total: report.total_outstanding,
            },
            InvoiceItemPdf {
                description: "Paid Invoices".to_string(),
                quantity: report.paid_invoices as f64,
                unit_price: 0.0,
                total: 0.0,
            },
            InvoiceItemPdf {
                description: "Overdue Invoices".to_string(),
                quantity: report.overdue_invoices as f64,
                unit_price: 0.0,
                total: 0.0,
            },
            InvoiceItemPdf {
                description: "Total Expenses".to_string(),
                quantity: 1.0,
                unit_price: report.total_expenses,
                total: report.total_expenses,
            },
            InvoiceItemPdf {
                description: "Net Profit".to_string(),
                quantity: 1.0,
                unit_price: report.net_profit,
                total: report.net_profit,
            },
        ];

        let pdf = self.pdf_service.generate_invoice_pdf(
            &format!("OVERVIEW-REPORT-{}-{}", start_date, end_date),
            Some("FlashBill"),
            Some("Overview Report"),
            "Report Generated",
            None,
            None,
            &start_date.to_string(),
            &end_date.to_string(),
            &items,
            report.total_revenue,
            0.0,
            0.0,
            report.net_profit,
            Some(&format!("Overview report from {} to {}", start_date, end_date)),
            None,
            None,
        )?;

        Ok(pdf)
    }
}
