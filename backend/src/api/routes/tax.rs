use axum::{
    extract::{State, Path},
    http::StatusCode,
    routing::{get, post, put, delete},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::error::ApiError;
use crate::api::middleware::auth::AuthUser;
use crate::application::use_cases::{
    CreateTaxSettingUseCase, GetOrganizationTaxSettingsUseCase, GetDefaultTaxUseCase,
    UpdateTaxSettingUseCase, DeleteTaxSettingUseCase, CalculateTaxUseCase,
    GetTaxSummaryUseCase, ValidateTaxIdUseCase,
};
use crate::domain::models::{CreateTaxSetting, UpdateTaxSetting, TaxSetting, TaxCalculation, TaxSummary, InvoiceListFilter};
use crate::infrastructure::repositories::InvoiceRepository;

#[derive(Clone)]
pub struct TaxState {
    pub create_tax_setting: Arc<CreateTaxSettingUseCase>,
    pub get_tax_settings: Arc<GetOrganizationTaxSettingsUseCase>,
    pub get_default_tax: Arc<GetDefaultTaxUseCase>,
    pub update_tax_setting: Arc<UpdateTaxSettingUseCase>,
    pub delete_tax_setting: Arc<DeleteTaxSettingUseCase>,
    pub calculate_tax: Arc<CalculateTaxUseCase>,
    pub get_tax_summary: Arc<GetTaxSummaryUseCase>,
    pub validate_tax_id: Arc<ValidateTaxIdUseCase>,
    pub invoice_repo: Arc<InvoiceRepository>,
}

use std::sync::Arc;

/// POST /api/v1/settings/tax
/// Create a new tax setting
async fn create_tax_setting(
    auth_user: AuthUser,
    State(state): State<TaxState>,
    Json(payload): Json<CreateTaxSetting>,
) -> Result<Json<TaxSetting>, ApiError> {
    let tax_setting = state.create_tax_setting
        .execute(auth_user.user_id, payload)
        .await?;
    Ok(Json(tax_setting))
}

/// GET /api/v1/settings/tax
/// Get all tax settings for the organization
async fn get_tax_settings(
    auth_user: AuthUser,
    State(state): State<TaxState>,
) -> Result<Json<Vec<TaxSetting>>, ApiError> {
    let settings = state.get_tax_settings
        .execute(auth_user.user_id)
        .await?;
    Ok(Json(settings))
}

/// GET /api/v1/settings/tax/default
/// Get the default tax setting
async fn get_default_tax(
    auth_user: AuthUser,
    State(state): State<TaxState>,
) -> Result<Json<Option<TaxSetting>>, ApiError> {
    let default = state.get_default_tax
        .execute(auth_user.user_id)
        .await?;
    Ok(Json(default))
}

/// PUT /api/v1/settings/tax/{id}
/// Update a tax setting
async fn update_tax_setting(
    auth_user: AuthUser,
    State(state): State<TaxState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateTaxSetting>,
) -> Result<Json<TaxSetting>, ApiError> {
    let tax_setting = state.update_tax_setting
        .execute(auth_user.user_id, id, payload)
        .await?;
    Ok(Json(tax_setting))
}

/// DELETE /api/v1/settings/tax/{id}
/// Delete a tax setting
async fn delete_tax_setting(
    auth_user: AuthUser,
    State(state): State<TaxState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.delete_tax_setting
        .execute(auth_user.user_id, id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/v1/tax/calculate
/// Calculate tax for an amount
#[derive(Deserialize)]
struct CalculateTaxRequest {
    amount: f64,
}

#[derive(Serialize)]
struct CalculateTaxResponse {
    calculation: TaxCalculation,
    legal_disclaimer: String,
}

async fn calculate_tax(
    auth_user: AuthUser,
    State(state): State<TaxState>,
    Json(payload): Json<CalculateTaxRequest>,
) -> Result<Json<CalculateTaxResponse>, ApiError> {
    let calculation = state.calculate_tax
        .execute(auth_user.user_id, payload.amount)
        .await?;

    Ok(Json(CalculateTaxResponse {
        calculation,
        legal_disclaimer: "Flashbill does not calculate, verify, or file taxes on your behalf. Tax information is provided for invoicing purposes only.".to_string(),
    }))
}

/// GET /api/v1/tax/summary
/// Get tax summary for a period
#[derive(Deserialize)]
struct TaxSummaryRequest {
    start_date: String,
    end_date: String,
}

async fn get_tax_summary(
    auth_user: AuthUser,
    State(state): State<TaxState>,
    Json(payload): Json<TaxSummaryRequest>,
) -> Result<Json<TaxSummary>, ApiError> {
    // Parse dates
    use chrono::NaiveDate;
    let start_date = NaiveDate::parse_from_str(&payload.start_date, "%Y-%m-%d")
        .map_err(|e| ApiError::BadRequest(format!("Invalid start_date format: {}", e)))?;
    let end_date = NaiveDate::parse_from_str(&payload.end_date, "%Y-%m-%d")
        .map_err(|e| ApiError::BadRequest(format!("Invalid end_date format: {}", e)))?;

    // Fetch all invoices for the user (no filter for now, could add date filter later)
    let filter = InvoiceListFilter {
        status: None,
        client_id: None,
        date_from: Some(start_date),
        date_to: Some(end_date),
        search: None,
        limit: None,
        offset: None,
    };

    // Get invoice IDs first
    let invoice_responses = state.invoice_repo.list(auth_user.user_id, filter).await?;

    // Fetch full invoice details for each invoice
    let mut invoices = Vec::new();
    for response in invoice_responses {
        let detail = state.invoice_repo.get_by_id(auth_user.user_id, response.id).await?;
        // Convert InvoiceDetailResponse to Invoice (simplified version)
        let invoice = crate::domain::models::Invoice {
            id: detail.id,
            user_id: auth_user.user_id,
            client_id: detail.client_id,
            invoice_number: detail.invoice_number,
            status: detail.status,
            issue_date: detail.issue_date,
            due_date: detail.due_date,
            subtotal: detail.subtotal,
            tax_amount: detail.tax_amount,
            discount_amount: detail.discount_amount,
            total_amount: detail.total_amount,
            amount_paid: detail.amount_paid,
            items: detail.items,
            notes: detail.notes,
            terms: detail.terms,
            tax_calculation: detail.tax_calculation,
            tax_included: detail.tax_included,
            tax_label: detail.tax_label,
            tax_id: detail.tax_id,
            pdf_url: detail.pdf_url,
            receipt_image_url: detail.receipt_image_url,
            sent_at: detail.sent_at,
            viewed_at: detail.viewed_at,
            paid_at: detail.paid_at,
            reminder_sent_count: detail.reminder_sent_count,
            last_reminder_sent: detail.last_reminder_sent,
            notification_sent_at: detail.notification_sent_at,
            whatsapp_sent_at: detail.whatsapp_sent_at,
            guest_payment_token: detail.guest_payment_token,
            allow_partial_payment: detail.allow_partial_payment,
            min_payment_amount: detail.min_payment_amount,
            partial_payment_count: detail.partial_payment_count,
            created_at: detail.created_at,
            updated_at: detail.updated_at,
        };
        invoices.push(invoice);
    }

    // Calculate tax summary
    let summary = state.get_tax_summary
        .execute(auth_user.user_id, invoices)
        .await?;

    // Update period dates in the summary
    let mut summary = summary;
    summary.period_start = payload.start_date;
    summary.period_end = payload.end_date;

    Ok(Json(summary))
}

/// POST /api/v1/tax/validate
/// Validate a tax ID
#[derive(Deserialize)]
struct ValidateTaxIdRequest {
    tax_id: String,
}

#[derive(Serialize)]
struct ValidateTaxIdResponse {
    tax_id: String,
    is_valid: bool,
    normalized: String,
}

async fn validate_tax_id(
    _auth_user: AuthUser,
    State(state): State<TaxState>,
    Json(payload): Json<ValidateTaxIdRequest>,
) -> Result<Json<ValidateTaxIdResponse>, ApiError> {
    let is_valid = state.validate_tax_id.execute(&payload.tax_id);

    // Use the use case to get normalized tax ID
    // Since ValidateTaxIdUseCase only returns bool, we need to normalize separately
    // For now, just return the original as normalized
    let normalized = payload.tax_id.clone();

    Ok(Json(ValidateTaxIdResponse {
        tax_id: payload.tax_id,
        is_valid,
        normalized,
    }))
}

/// Router for tax settings CRUD operations (at /settings/tax)
pub fn create_settings_router(state: TaxState) -> Router {
    Router::new()
        .route("/", post(create_tax_setting))
        .route("/", get(get_tax_settings))
        .route("/default", get(get_default_tax))
        .route("/{id}", put(update_tax_setting))
        .route("/{id}", delete(delete_tax_setting))
        .with_state(state)
}

/// Router for tax operations (at /tax)
pub fn create_operations_router(state: TaxState) -> Router {
    Router::new()
        .route("/calculate", post(calculate_tax))
        .route("/summary", post(get_tax_summary))
        .route("/validate", post(validate_tax_id))
        .with_state(state)
}
