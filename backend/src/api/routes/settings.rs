use axum::{
    routing::{get, put},
    extract::State,
    Json, Router,
};
use std::sync::Arc;

use crate::api::error::ApiError;
use crate::api::middleware::AuthUser;
use crate::domain::models::{BusinessAddress, TaxSettings, NotificationSettings, InvoiceSettings};
use crate::application::use_cases::{
    GetBusinessSettingsUseCase, UpdateBusinessSettingsUseCase,
    GetTaxSettingsUseCase, UpdateTaxSettingsUseCase,
    GetNotificationSettingsUseCase, UpdateNotificationSettingsUseCase,
    GetInvoiceSettingsUseCase, UpdateInvoiceSettingsUseCase,
};

#[derive(Clone)]
struct SettingsState {
    get_business_uc: Arc<GetBusinessSettingsUseCase>,
    update_business_uc: Arc<UpdateBusinessSettingsUseCase>,
    get_tax_uc: Arc<GetTaxSettingsUseCase>,
    update_tax_uc: Arc<UpdateTaxSettingsUseCase>,
    get_notification_uc: Arc<GetNotificationSettingsUseCase>,
    update_notification_uc: Arc<UpdateNotificationSettingsUseCase>,
    get_invoice_uc: Arc<GetInvoiceSettingsUseCase>,
    update_invoice_uc: Arc<UpdateInvoiceSettingsUseCase>,
}

pub fn create_router(
    get_business_uc: Arc<GetBusinessSettingsUseCase>,
    update_business_uc: Arc<UpdateBusinessSettingsUseCase>,
    get_tax_uc: Arc<GetTaxSettingsUseCase>,
    update_tax_uc: Arc<UpdateTaxSettingsUseCase>,
    get_notification_uc: Arc<GetNotificationSettingsUseCase>,
    update_notification_uc: Arc<UpdateNotificationSettingsUseCase>,
    get_invoice_uc: Arc<GetInvoiceSettingsUseCase>,
    update_invoice_uc: Arc<UpdateInvoiceSettingsUseCase>,
) -> Router {
    let state = SettingsState {
        get_business_uc,
        update_business_uc,
        get_tax_uc,
        update_tax_uc,
        get_notification_uc,
        update_notification_uc,
        get_invoice_uc,
        update_invoice_uc,
    };

    Router::new()
        .route("/business", get(get_business_settings))
        .route("/business", put(update_business_settings))
        .route("/tax", get(get_tax_settings))
        .route("/tax", put(update_tax_settings))
        .route("/notifications", get(get_notification_settings))
        .route("/notifications", put(update_notification_settings))
        .route("/invoice", get(get_invoice_settings))
        .route("/invoice", put(update_invoice_settings))
        .with_state(state)
}

#[derive(serde::Serialize)]
struct BusinessSettingsResponse {
    company_name: Option<String>,
    business_type: Option<String>,
    address: Option<BusinessAddress>,
    phone: Option<String>,
    email: String,
}

async fn get_business_settings(
    auth_user: AuthUser,
    State(state): State<SettingsState>,
) -> Result<Json<BusinessSettingsResponse>, ApiError> {
    let user = state.get_business_uc.execute(auth_user.user_id).await?;
    Ok(Json(BusinessSettingsResponse {
        company_name: user.company_name,
        business_type: user.business_type,
        address: user.business_address,
        phone: user.phone,
        email: user.email,
    }))
}

#[derive(serde::Deserialize)]
struct UpdateBusinessRequest {
    company_name: Option<String>,
    business_type: Option<String>,
    address: Option<BusinessAddress>,
    phone: Option<String>,
}

async fn update_business_settings(
    auth_user: AuthUser,
    State(state): State<SettingsState>,
    Json(payload): Json<UpdateBusinessRequest>,
) -> Result<Json<BusinessSettingsResponse>, ApiError> {
    let user = state.update_business_uc.execute(
        auth_user.user_id,
        payload.company_name,
        payload.business_type,
        payload.address,
        payload.phone,
    ).await?;

    Ok(Json(BusinessSettingsResponse {
        company_name: user.company_name,
        business_type: user.business_type,
        address: user.business_address,
        phone: user.phone,
        email: user.email,
    }))
}

#[derive(serde::Serialize)]
struct TaxSettingsResponse {
    tax_settings: TaxSettings,
    available_states: Vec<String>,
}

async fn get_tax_settings(
    auth_user: AuthUser,
    State(state): State<SettingsState>,
) -> Result<Json<TaxSettingsResponse>, ApiError> {
    let user = state.get_tax_uc.execute(auth_user.user_id).await?;
    let tax_settings = user.tax_settings.unwrap_or_else(|| TaxSettings {
        state_code: "CA".to_string(),
        tax_rate: 7.25,
        tax_exempt: false,
        tax_id: None,
    });

    Ok(Json(TaxSettingsResponse {
        tax_settings,
        available_states: vec!["CA".to_string(), "NY".to_string(), "TX".to_string(), "FL".to_string()],
    }))
}

async fn update_tax_settings(
    auth_user: AuthUser,
    State(state): State<SettingsState>,
    Json(payload): Json<TaxSettings>,
) -> Result<Json<TaxSettings>, ApiError> {
    let user = state.update_tax_uc.execute(auth_user.user_id, payload.clone()).await?;
    Ok(Json(user.tax_settings.unwrap_or(payload)))
}

async fn get_notification_settings(
    auth_user: AuthUser,
    State(state): State<SettingsState>,
) -> Result<Json<NotificationSettings>, ApiError> {
    let user = state.get_notification_uc.execute(auth_user.user_id).await?;
    Ok(Json(user.notification_settings))
}

async fn update_notification_settings(
    auth_user: AuthUser,
    State(state): State<SettingsState>,
    Json(payload): Json<NotificationSettings>,
) -> Result<Json<NotificationSettings>, ApiError> {
    let user = state.update_notification_uc.execute(auth_user.user_id, payload.clone()).await?;
    Ok(Json(user.notification_settings))
}

#[derive(serde::Serialize)]
struct InvoiceSettingsResponse {
    template: String,
    logo_url: Option<String>,
    terms: String,
    notes: String,
}

async fn get_invoice_settings(
    auth_user: AuthUser,
    State(state): State<SettingsState>,
) -> Result<Json<InvoiceSettingsResponse>, ApiError> {
    let user = state.get_invoice_uc.execute(auth_user.user_id).await?;
    let invoice_settings = user.invoice_settings.unwrap_or_default();

    Ok(Json(InvoiceSettingsResponse {
        template: invoice_settings.template,
        logo_url: invoice_settings.logo_url,
        terms: invoice_settings.terms,
        notes: invoice_settings.notes,
    }))
}

#[derive(serde::Deserialize)]
struct UpdateInvoiceRequest {
    template: String,
    logo_url: Option<String>,
    terms: String,
    notes: String,
}

async fn update_invoice_settings(
    auth_user: AuthUser,
    State(state): State<SettingsState>,
    Json(payload): Json<UpdateInvoiceRequest>,
) -> Result<Json<InvoiceSettingsResponse>, ApiError> {
    let user = state.update_invoice_uc.execute(
        auth_user.user_id,
        payload.template.clone(),
        payload.logo_url.clone(),
        payload.terms.clone(),
        payload.notes.clone(),
    ).await?;

    let invoice_settings = user.invoice_settings.unwrap_or_default();

    Ok(Json(InvoiceSettingsResponse {
        template: invoice_settings.template,
        logo_url: invoice_settings.logo_url,
        terms: invoice_settings.terms,
        notes: invoice_settings.notes,
    }))
}
