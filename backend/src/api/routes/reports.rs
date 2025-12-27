use axum::{
    extract::{Query, State},
    http::HeaderMap,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use std::sync::Arc;
use chrono::NaiveDate;
use serde::Deserialize;

use crate::api::error::ApiError;
use crate::api::middleware::AuthUser;
use crate::application::use_cases::{
    GetOverviewStatsUseCase, GetIncomeReportUseCase, GetExpensesReportUseCase,
    GetTaxReportUseCase, GetAgingReportUseCase, ExportReportUseCase,
};
use crate::domain::repositories::report_repository::{
    OverviewStats, IncomeReport, ExpensesReport, TaxReport, AgingReport,
};

#[derive(Clone)]
struct ReportState {
    get_overview_stats_uc: Arc<GetOverviewStatsUseCase>,
    get_income_report_uc: Arc<GetIncomeReportUseCase>,
    get_expenses_report_uc: Arc<GetExpensesReportUseCase>,
    get_tax_report_uc: Arc<GetTaxReportUseCase>,
    get_aging_report_uc: Arc<GetAgingReportUseCase>,
    export_report_uc: Arc<ExportReportUseCase>,
}

pub fn create_router(
    get_overview_stats_uc: Arc<GetOverviewStatsUseCase>,
    get_income_report_uc: Arc<GetIncomeReportUseCase>,
    get_expenses_report_uc: Arc<GetExpensesReportUseCase>,
    get_tax_report_uc: Arc<GetTaxReportUseCase>,
    get_aging_report_uc: Arc<GetAgingReportUseCase>,
    export_report_uc: Arc<ExportReportUseCase>,
) -> Router {
    let state = ReportState {
        get_overview_stats_uc,
        get_income_report_uc,
        get_expenses_report_uc,
        get_tax_report_uc,
        get_aging_report_uc,
        export_report_uc,
    };

    Router::new()
        .route("/overview", get(get_overview))
        .route("/income", get(get_income_report))
        .route("/expenses", get(get_expenses_report))
        .route("/tax", get(get_tax_report))
        .route("/aging", get(get_aging_report))
        .route("/export", post(export_report))
        .with_state(state)
}

async fn get_overview(
    auth_user: AuthUser,
    State(state): State<ReportState>,
) -> Result<Json<OverviewStats>, ApiError> {
    let stats = state.get_overview_stats_uc.execute(auth_user.user_id).await?;
    Ok(Json(stats))
}

#[derive(Deserialize)]
struct DateRange {
    start_date: String,
    end_date: String,
}

async fn get_income_report(
    auth_user: AuthUser,
    State(state): State<ReportState>,
    Query(date_range): Query<DateRange>,
) -> Result<Json<IncomeReport>, ApiError> {
    let start_date = NaiveDate::parse_from_str(&date_range.start_date, "%Y-%m-%d")
        .map_err(|e| ApiError::BadRequest(format!("Invalid start_date: {}", e)))?;
    let end_date = NaiveDate::parse_from_str(&date_range.end_date, "%Y-%m-%d")
        .map_err(|e| ApiError::BadRequest(format!("Invalid end_date: {}", e)))?;

    let report = state.get_income_report_uc.execute(auth_user.user_id, start_date, end_date).await?;
    Ok(Json(report))
}

async fn get_expenses_report(
    auth_user: AuthUser,
    State(state): State<ReportState>,
    Query(date_range): Query<DateRange>,
) -> Result<Json<ExpensesReport>, ApiError> {
    let start_date = NaiveDate::parse_from_str(&date_range.start_date, "%Y-%m-%d")
        .map_err(|e| ApiError::BadRequest(format!("Invalid start_date: {}", e)))?;
    let end_date = NaiveDate::parse_from_str(&date_range.end_date, "%Y-%m-%d")
        .map_err(|e| ApiError::BadRequest(format!("Invalid end_date: {}", e)))?;

    let report = state.get_expenses_report_uc.execute(auth_user.user_id, start_date, end_date).await?;
    Ok(Json(report))
}

async fn get_tax_report(
    auth_user: AuthUser,
    State(state): State<ReportState>,
    Query(date_range): Query<DateRange>,
) -> Result<Json<TaxReport>, ApiError> {
    let start_date = NaiveDate::parse_from_str(&date_range.start_date, "%Y-%m-%d")
        .map_err(|e| ApiError::BadRequest(format!("Invalid start_date: {}", e)))?;
    let end_date = NaiveDate::parse_from_str(&date_range.end_date, "%Y-%m-%d")
        .map_err(|e| ApiError::BadRequest(format!("Invalid end_date: {}", e)))?;

    let report = state.get_tax_report_uc.execute(auth_user.user_id, start_date, end_date).await?;
    Ok(Json(report))
}

async fn get_aging_report(
    auth_user: AuthUser,
    State(state): State<ReportState>,
) -> Result<Json<AgingReport>, ApiError> {
    let report = state.get_aging_report_uc.execute(auth_user.user_id).await?;
    Ok(Json(report))
}

#[derive(Deserialize)]
struct ExportRequest {
    report_type: String,
    format: String, // "pdf" or "csv"
    date_range: DateRange,
}

async fn export_report(
    auth_user: AuthUser,
    State(state): State<ReportState>,
    Json(payload): Json<ExportRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let start_date = NaiveDate::parse_from_str(&payload.date_range.start_date, "%Y-%m-%d")
        .map_err(|e| ApiError::BadRequest(format!("Invalid start_date: {}", e)))?;
    let end_date = NaiveDate::parse_from_str(&payload.date_range.end_date, "%Y-%m-%d")
        .map_err(|e| ApiError::BadRequest(format!("Invalid end_date: {}", e)))?;

    let file_data = state.export_report_uc.execute(
        auth_user.user_id,
        payload.report_type.clone(),
        payload.format.clone(),
        start_date,
        end_date,
    ).await?;

    // Generate filename
    let filename = format!("{}_{}_{}_{}.{}",
        payload.report_type,
        start_date,
        end_date,
        chrono::Utc::now().timestamp(),
        payload.format
    );

    // Determine content type
    let content_type = if payload.format == "pdf" {
        "application/pdf"
    } else {
        "text/csv"
    };

    // Create response with file download headers
    let mut headers = HeaderMap::new();
    headers.insert(
        axum::http::header::CONTENT_TYPE,
        content_type.parse().unwrap(),
    );
    headers.insert(
        axum::http::header::CONTENT_DISPOSITION,
        format!("attachment; filename=\"{}\"", filename).parse().unwrap(),
    );

    Ok((headers, file_data))
}
