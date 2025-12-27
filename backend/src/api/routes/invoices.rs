use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post, put, delete},
    Json, Router,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::api::error::ApiError;
use crate::api::middleware::AuthUser;
use crate::application::dto::invoice_dto::*;
use crate::application::use_cases::*;

#[derive(Clone)]
struct InvoiceState {
    create_invoice_uc: Arc<CreateInvoiceUseCase>,
    get_invoice_uc: Arc<GetInvoiceUseCase>,
    list_invoices_uc: Arc<ListInvoicesUseCase>,
    update_invoice_uc: Arc<UpdateInvoiceUseCase>,
    delete_invoice_uc: Arc<DeleteInvoiceUseCase>,
    record_payment_uc: Arc<RecordPaymentUseCase>,
    send_invoice_uc: Arc<SendInvoiceUseCase>,
    get_pdf_uc: Arc<GetInvoicePdfUseCase>,
    send_reminder_uc: Arc<SendReminderUseCase>,
    send_invoice_whatsapp_uc: Arc<SendInvoiceWhatsappUseCase>,
    mark_invoice_viewed_uc: Arc<MarkInvoiceViewedUseCase>,
    send_payment_confirmation_uc: Arc<SendPaymentConfirmationUseCase>,
    add_discussion_message_uc: Arc<AddDiscussionMessageUseCase>,
    get_discussion_messages_uc: Arc<GetDiscussionMessagesUseCase>,
}

pub fn create_router(
    create_invoice_uc: Arc<CreateInvoiceUseCase>,
    get_invoice_uc: Arc<GetInvoiceUseCase>,
    list_invoices_uc: Arc<ListInvoicesUseCase>,
    update_invoice_uc: Arc<UpdateInvoiceUseCase>,
    delete_invoice_uc: Arc<DeleteInvoiceUseCase>,
    record_payment_uc: Arc<RecordPaymentUseCase>,
    send_invoice_uc: Arc<SendInvoiceUseCase>,
    get_pdf_uc: Arc<GetInvoicePdfUseCase>,
    send_reminder_uc: Arc<SendReminderUseCase>,
    send_invoice_whatsapp_uc: Arc<SendInvoiceWhatsappUseCase>,
    mark_invoice_viewed_uc: Arc<MarkInvoiceViewedUseCase>,
    send_payment_confirmation_uc: Arc<SendPaymentConfirmationUseCase>,
    add_discussion_message_uc: Arc<AddDiscussionMessageUseCase>,
    get_discussion_messages_uc: Arc<GetDiscussionMessagesUseCase>,
) -> Router {
    let state = InvoiceState {
        create_invoice_uc,
        get_invoice_uc,
        list_invoices_uc,
        update_invoice_uc,
        delete_invoice_uc,
        record_payment_uc,
        send_invoice_uc,
        get_pdf_uc,
        send_reminder_uc,
        send_invoice_whatsapp_uc,
        mark_invoice_viewed_uc,
        send_payment_confirmation_uc,
        add_discussion_message_uc,
        get_discussion_messages_uc,
    };

    Router::new()
        .route("/", get(list_invoices))
        .route("/", post(create_invoice))
        .route("/{id}", get(get_invoice))
        .route("/{id}", put(update_invoice))
        .route("/{id}", delete(delete_invoice))
        .route("/{id}/send", post(send_invoice))
        .route("/{id}/send-whatsapp", post(send_invoice_whatsapp))
        .route("/{id}/remind", post(send_reminder))
        .route("/{id}/pdf", get(get_pdf))
        .route("/{id}/pay", post(record_payment))
        .route("/{id}/view", post(mark_invoice_viewed))
        .route("/{id}/send-confirmation", post(send_payment_confirmation))
        .route("/{id}/discussion", get(get_discussion_messages))
        .route("/{id}/discussion", post(add_discussion_message))
        .with_state(state)
}

async fn list_invoices(
    auth_user: AuthUser,
    State(state): State<InvoiceState>,
    Query(query): Query<InvoiceListQuery>,
) -> Result<Json<Vec<InvoiceSummaryDto>>, ApiError> {
    let invoices = state
        .list_invoices_uc
        .execute(auth_user.user_id, query)
        .await?;

    Ok(Json(invoices))
}

async fn create_invoice(
    auth_user: AuthUser,
    State(state): State<InvoiceState>,
    Json(payload): Json<CreateInvoiceCommand>,
) -> Result<(StatusCode, Json<InvoiceCreatedDto>), ApiError> {
    let response = state
        .create_invoice_uc
        .execute(auth_user.user_id, payload)
        .await?;

    Ok((StatusCode::CREATED, Json(response)))
}

async fn get_invoice(
    auth_user: AuthUser,
    State(state): State<InvoiceState>,
    Path(invoice_id): Path<Uuid>,
) -> Result<Json<InvoiceDto>, ApiError> {
    let response = state
        .get_invoice_uc
        .execute(auth_user.user_id, invoice_id)
        .await?;

    Ok(Json(response))
}

async fn update_invoice(
    auth_user: AuthUser,
    State(state): State<InvoiceState>,
    Path(invoice_id): Path<Uuid>,
    Json(payload): Json<UpdateInvoiceCommand>,
) -> Result<Json<InvoiceDto>, ApiError> {
    let response = state
        .update_invoice_uc
        .execute(auth_user.user_id, invoice_id, payload)
        .await?;

    Ok(Json(response))
}

async fn delete_invoice(
    auth_user: AuthUser,
    State(state): State<InvoiceState>,
    Path(invoice_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state
        .delete_invoice_uc
        .execute(auth_user.user_id, invoice_id)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

async fn send_invoice(
    auth_user: AuthUser,
    State(state): State<InvoiceState>,
    Path(invoice_id): Path<Uuid>,
    Json(payload): Json<SendInvoiceCommand>,
) -> Result<StatusCode, ApiError> {
    state
        .send_invoice_uc
        .execute(auth_user.user_id, invoice_id, payload)
        .await?;

    Ok(StatusCode::OK)
}

async fn send_reminder(
    auth_user: AuthUser,
    State(state): State<InvoiceState>,
    Path(invoice_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state
        .send_reminder_uc
        .execute(auth_user.user_id, invoice_id)
        .await?;

    Ok(StatusCode::OK)
}

async fn get_pdf(
    auth_user: AuthUser,
    State(state): State<InvoiceState>,
    Path(invoice_id): Path<Uuid>,
) -> Result<Vec<u8>, ApiError> {
    let pdf_bytes = state
        .get_pdf_uc
        .execute(auth_user.user_id, invoice_id)
        .await?;

    Ok(pdf_bytes)
}

async fn record_payment(
    auth_user: AuthUser,
    State(state): State<InvoiceState>,
    Path(invoice_id): Path<Uuid>,
    Json(payload): Json<RecordPaymentCommand>,
) -> Result<(StatusCode, Json<PaymentRecordedDto>), ApiError> {
    let response = state
        .record_payment_uc
        .execute(auth_user.user_id, invoice_id, payload)
        .await?;

    Ok((StatusCode::CREATED, Json(response)))
}

async fn send_invoice_whatsapp(
    auth_user: AuthUser,
    State(state): State<InvoiceState>,
    Path(invoice_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state
        .send_invoice_whatsapp_uc
        .execute(auth_user.user_id, invoice_id)
        .await?;

    Ok(StatusCode::OK)
}

async fn mark_invoice_viewed(
    State(state): State<InvoiceState>,
    Path(invoice_id): Path<Uuid>,
) -> Result<Json<InvoiceDto>, ApiError> {
    let response = state
        .mark_invoice_viewed_uc
        .execute(invoice_id)
        .await?;

    Ok(Json(response))
}

async fn send_payment_confirmation(
    auth_user: AuthUser,
    State(state): State<InvoiceState>,
    Path(invoice_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state
        .send_payment_confirmation_uc
        .execute(auth_user.user_id, invoice_id)
        .await?;

    Ok(StatusCode::OK)
}

async fn add_discussion_message(
    auth_user: AuthUser,
    State(state): State<InvoiceState>,
    Path(invoice_id): Path<Uuid>,
    Json(payload): Json<AddDiscussionMessageCommand>,
) -> Result<(StatusCode, Json<DiscussionMessageDto>), ApiError> {
    let response = state
        .add_discussion_message_uc
        .execute_as_seller(auth_user.user_id, invoice_id, payload)
        .await?;

    Ok((StatusCode::CREATED, Json(response)))
}

async fn get_discussion_messages(
    auth_user: AuthUser,
    State(state): State<InvoiceState>,
    Path(invoice_id): Path<Uuid>,
) -> Result<Json<DiscussionResponseDto>, ApiError> {
    let response = state
        .get_discussion_messages_uc
        .execute(auth_user.user_id, invoice_id)
        .await?;

    Ok(Json(response))
}
