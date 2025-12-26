use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::api::error::ApiError;
use crate::api::middleware::AuthUser;
use crate::domain::models::{CreatePayment, PaymentListFilter, RefundRequest, PaymentStatus, PaymentMethod};
use crate::application::use_cases::{
    CreatePaymentUseCase, GetPaymentUseCase, ListPaymentsUseCase,
    RefundPaymentUseCase, GetPaymentStatsUseCase, GetPaymentMethodsUseCase,
};

#[derive(Clone)]
struct PaymentState {
    create_payment_uc: Arc<CreatePaymentUseCase>,
    get_payment_uc: Arc<GetPaymentUseCase>,
    list_payments_uc: Arc<ListPaymentsUseCase>,
    refund_payment_uc: Arc<RefundPaymentUseCase>,
    get_payment_stats_uc: Arc<GetPaymentStatsUseCase>,
    get_payment_methods_uc: Arc<GetPaymentMethodsUseCase>,
}

pub fn create_router(
    create_payment_uc: Arc<CreatePaymentUseCase>,
    get_payment_uc: Arc<GetPaymentUseCase>,
    list_payments_uc: Arc<ListPaymentsUseCase>,
    refund_payment_uc: Arc<RefundPaymentUseCase>,
    get_payment_stats_uc: Arc<GetPaymentStatsUseCase>,
    get_payment_methods_uc: Arc<GetPaymentMethodsUseCase>,
) -> Router {
    let state = PaymentState {
        create_payment_uc,
        get_payment_uc,
        list_payments_uc,
        refund_payment_uc,
        get_payment_stats_uc,
        get_payment_methods_uc,
    };

    Router::new()
        .route("/", get(list_payments))
        .route("/", post(create_payment))
        .route("/{id}", get(get_payment))
        .route("/{id}/refund", post(refund_payment))
        .route("/stats", get(get_payment_stats))
        .route("/methods", get(get_payment_methods))
        .with_state(state)
}

async fn list_payments(
    auth_user: AuthUser,
    State(state): State<PaymentState>,
    Query(filter): Query<PaymentListFilter>,
) -> Result<Json<Vec<crate::domain::models::PaymentResponse>>, ApiError> {
    let payments = state.list_payments_uc.execute(
        auth_user.user_id,
        filter.status,
        filter.payment_method,
        filter.date_from,
        filter.date_to,
        filter.limit,
        filter.offset,
    ).await?;
    Ok(Json(payments))
}

async fn create_payment(
    auth_user: AuthUser,
    State(state): State<PaymentState>,
    Json(payload): Json<CreatePayment>,
) -> Result<(StatusCode, Json<crate::domain::models::Payment>), ApiError> {
    let payment = state.create_payment_uc.execute(auth_user.user_id, payload).await?;
    Ok((StatusCode::CREATED, Json(payment)))
}

async fn get_payment(
    auth_user: AuthUser,
    State(state): State<PaymentState>,
    Path(payment_id): Path<Uuid>,
) -> Result<Json<crate::domain::models::PaymentResponse>, ApiError> {
    let payment = state.get_payment_uc.execute(auth_user.user_id, payment_id).await?;
    Ok(Json(payment))
}

async fn refund_payment(
    auth_user: AuthUser,
    State(state): State<PaymentState>,
    Path(payment_id): Path<Uuid>,
    Json(payload): Json<RefundRequest>,
) -> Result<Json<crate::domain::models::Payment>, ApiError> {
    let refund = state.refund_payment_uc.execute(auth_user.user_id, payment_id, payload).await?;
    Ok(Json(refund))
}

async fn get_payment_methods(
    _auth_user: AuthUser,
    State(state): State<PaymentState>,
) -> Result<Json<Vec<String>>, ApiError> {
    let methods = state.get_payment_methods_uc.execute();
    Ok(Json(methods))
}

async fn get_payment_stats(
    auth_user: AuthUser,
    State(state): State<PaymentState>,
) -> Result<Json<crate::domain::models::PaymentStats>, ApiError> {
    let stats = state.get_payment_stats_uc.execute(auth_user.user_id).await?;
    Ok(Json(stats))
}
