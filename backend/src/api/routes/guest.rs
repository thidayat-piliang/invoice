use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::api::error::ApiError;
use crate::application::dto::invoice_dto::{AddDiscussionMessageCommand, DiscussionMessageDto, DiscussionResponseDto};
use crate::domain::models::invoice::{InvoiceDetailResponse, InvoiceStatus};
use crate::domain::models::payment::{CreatePayment, PaymentMethod};
use crate::domain::services::invoice_service::InvoiceService;
use crate::domain::services::notification_service_new::EnhancedNotificationService;
use crate::domain::services::payment_gateway_service::{CreatePaymentIntent, PaymentGatewayService};
use crate::infrastructure::repositories::invoice_repository::InvoiceRepository;
use crate::infrastructure::repositories::payment_repository::PaymentRepository;
use crate::infrastructure::repositories::user_repository::UserRepository;

#[derive(Clone)]
pub struct GuestState {
    pub invoice_repo: Arc<InvoiceRepository>,
    pub payment_repo: Arc<PaymentRepository>,
    pub user_repo: Arc<UserRepository>,
    pub invoice_service: Arc<InvoiceService>,
    pub payment_gateway: Arc<PaymentGatewayService>,
    pub notification_service: Arc<EnhancedNotificationService>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GuestInvoiceResponse {
    pub invoice: InvoiceDetailResponse,
    pub seller: GuestSellerInfo,
    pub payment_methods: Vec<String>,
    pub guest_payment_link: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GuestSellerInfo {
    pub company_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GuestPaymentRequest {
    pub amount: f64,
    pub payment_method: PaymentMethod,
    pub customer_email: Option<String>,
    pub customer_phone: Option<String>,
    pub customer_name: String,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GuestPaymentResponse {
    pub payment_id: Uuid,
    pub status: String,
    pub redirect_url: Option<String>,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GuestPaymentHistoryRequest {
    pub email: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GuestPaymentHistoryResponse {
    pub payments: Vec<GuestPaymentSummary>,
    pub total_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GuestPaymentSummary {
    pub invoice_number: String,
    pub amount: f64,
    pub paid_at: chrono::DateTime<Utc>,
    pub status: String,
}

/// Get invoice by token (guest access)
async fn get_invoice_by_token(
    State(state): State<GuestState>,
    Path(token): Path<String>,
) -> Result<(StatusCode, Json<GuestInvoiceResponse>), ApiError> {
    // Verify token (simplified - in production use JWT or signed tokens)
    // For now, we'll decode the token to get invoice_id
    // Format: "guest_{invoice_id}_{hash}"

    let parts: Vec<&str> = token.split('_').collect();
    if parts.len() < 2 {
        return Err(ApiError::BadRequest("Invalid token format".to_string()));
    }

    let invoice_id_str = parts[1];
    let invoice_id = Uuid::parse_str(invoice_id_str)
        .map_err(|_| ApiError::BadRequest("Invalid invoice ID".to_string()))?;

    // Get invoice
    let invoice = state
        .invoice_repo
        .get_invoice_by_id(invoice_id)
        .await
        .map_err(|_| ApiError::NotFound)?;

    // Get seller info
    let seller = state
        .user_repo
        .find_by_id(invoice.user_id)
        .await
        .map_err(|_| ApiError::Database("Database error".to_string()))?
        .ok_or(ApiError::NotFound)?;

    // Get available payment methods
    let payment_methods = state.payment_gateway.get_available_gateways();

    // Generate guest payment link
    let guest_payment_link = format!(
        "https://yourapp.com/guest/pay/{}",
        token
    );

    let response = GuestInvoiceResponse {
        invoice,
        seller: GuestSellerInfo {
            company_name: seller.company_name,
            email: Some(seller.email),
            phone: seller.phone,
        },
        payment_methods,
        guest_payment_link,
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Process guest payment
async fn process_guest_payment(
    State(state): State<GuestState>,
    Path(token): Path<String>,
    Json(payload): Json<GuestPaymentRequest>,
) -> Result<(StatusCode, Json<GuestPaymentResponse>), ApiError> {
    // Verify token
    let parts: Vec<&str> = token.split('_').collect();
    if parts.len() < 2 {
        return Err(ApiError::BadRequest("Invalid token format".to_string()));
    }

    let invoice_id_str = parts[1];
    let invoice_id = Uuid::parse_str(invoice_id_str)
        .map_err(|_| ApiError::BadRequest("Invalid invoice ID".to_string()))?;

    // Get invoice
    let invoice = state
        .invoice_repo
        .get_invoice_by_id(invoice_id)
        .await
        .map_err(|_| ApiError::NotFound)?;

    // Check if already paid
    if invoice.status == InvoiceStatus::Paid {
        return Err(ApiError::BadRequest("Invoice already paid".to_string()));
    }

    // Check if amount matches
    if payload.amount != invoice.total_amount {
        return Err(ApiError::BadRequest(
            "Payment amount does not match invoice total".to_string(),
        ));
    }

    // Process payment based on method
    let payment_result = match payload.payment_method {
        PaymentMethod::PayPal => {
            let intent = CreatePaymentIntent {
                amount: payload.amount,
                currency: "USD".to_string(),
                description: Some(format!("Payment for invoice {}", invoice.invoice_number)),
                metadata: None,
            };
            state.payment_gateway.create_paypal_order(intent).await
        }
        PaymentMethod::Stripe => {
            let intent = CreatePaymentIntent {
                amount: payload.amount,
                currency: "USD".to_string(),
                description: Some(format!("Payment for invoice {}", invoice.invoice_number)),
                metadata: None,
            };
            state.payment_gateway.create_stripe_payment_intent(intent).await
        }
        PaymentMethod::AchDebit => {
            let intent = CreatePaymentIntent {
                amount: payload.amount,
                currency: "USD".to_string(),
                description: Some(format!("Payment for invoice {}", invoice.invoice_number)),
                metadata: None,
            };
            state.payment_gateway.create_ach_payment(intent).await
        }
        PaymentMethod::BankTransfer => {
            let intent = CreatePaymentIntent {
                amount: payload.amount,
                currency: "USD".to_string(),
                description: Some(format!("Payment for invoice {}", invoice.invoice_number)),
                metadata: None,
            };
            state.payment_gateway.create_bank_transfer_payment(intent).await
        }
        _ => {
            return Err(ApiError::BadRequest(
                "Payment method not supported for guest checkout".to_string(),
            ));
        }
    }?;

    // Create payment record
    let create_payment = CreatePayment {
        invoice_id,
        amount: payload.amount,
        payment_method: payload.payment_method.clone(),
        gateway: Some(payment_result.payment_method.clone()),
        gateway_payment_id: Some(payment_result.id.clone()),
        gateway_fee: Some(0.0),
        paid_by: Some(payload.customer_name.clone()),
        notes: payload.notes.clone(),
    };

    let payment = state
        .payment_repo
        .create_payment(create_payment)
        .await
        .map_err(|e| ApiError::Database(e.to_string()))?;

    // Auto-flag as paid if completed
    if payment_result.status == "completed" || payment_result.status == "succeeded" {
        // Convert Payment to CreatePayment
        let create_payment = CreatePayment {
            invoice_id,
            amount: payment.amount,
            payment_method: payment.payment_method.clone(),
            gateway: payment.gateway.clone(),
            gateway_payment_id: payment.gateway_payment_id.clone(),
            gateway_fee: Some(payment.gateway_fee),
            paid_by: payment.paid_by.clone(),
            notes: payment.notes.clone(),
        };

        let _updated_invoice = state
            .invoice_repo
            .record_payment_guest(invoice_id, create_payment)
            .await
            .map_err(|e| ApiError::Database(e.to_string()))?;

        // Get full invoice details for notification
        let invoice_detail = state
            .invoice_repo
            .get_invoice_by_id(invoice_id)
            .await
            .map_err(|e| ApiError::Database(e.to_string()))?;

        // Send payment confirmation
        let _ = state
            .notification_service
            .send_payment_confirmation(
                &invoice_detail,
                payload.customer_email.clone(),
                payload.customer_phone.clone(),
            )
            .await;
    }

    // Generate redirect URL based on payment method
    let redirect_url = match payload.payment_method {
        PaymentMethod::PayPal => Some(format!("https://paypal.com/checkout/{}", payment_result.id)),
        PaymentMethod::Stripe => Some(format!("https://stripe.com/pay/{}", payment_result.id)),
        PaymentMethod::AchDebit => Some(format!("https://yourapp.com/guest/ach/status/{}", payment.id)),
        PaymentMethod::BankTransfer => Some(format!("https://yourapp.com/guest/bt/status/{}", payment.id)),
        _ => None,
    };

    let response = GuestPaymentResponse {
        payment_id: payment.id,
        status: payment_result.status,
        redirect_url,
        message: "Payment initiated successfully".to_string(),
    };

    Ok((StatusCode::CREATED, Json(response)))
}

/// Get guest payment history (by email or phone)
async fn get_guest_payment_history(
    _state: State<GuestState>,
    Json(payload): Json<GuestPaymentHistoryRequest>,
) -> Result<(StatusCode, Json<GuestPaymentHistoryResponse>), ApiError> {
    // In production, you would verify identity via OTP or other means
    // For now, we'll just return mock data or require email/phone match

    if payload.email.is_none() && payload.phone.is_none() {
        return Err(ApiError::BadRequest(
            "Email or phone required".to_string(),
        ));
    }

    // Get recent payments (last 5)
    // This would query payments table filtered by customer email/phone
    // For now, return empty list (implementation depends on how customer info is stored)

    let payments = vec
![];

    // In production, implement:
    // 1. Find all invoices where client_email or client_phone matches
    // 2. Get payments for those invoices
    // 3. Return last 5

    let response = GuestPaymentHistoryResponse {
        payments,
        total_count: 0,
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Mark invoice as viewed (for tracking)
async fn mark_invoice_viewed(
    State(state): State<GuestState>,
    Path(token): Path<String>,
) -> Result<StatusCode, ApiError> {
    let parts: Vec<&str> = token.split('_').collect();
    if parts.len() < 2 {
        return Err(ApiError::BadRequest("Invalid token format".to_string()));
    }

    let invoice_id_str = parts[1];
    let invoice_id = Uuid::parse_str(invoice_id_str)
        .map_err(|_| ApiError::BadRequest("Invalid invoice ID".to_string()))?;

    // Update viewed_at timestamp
    state
        .invoice_repo
        .mark_as_viewed(invoice_id)
        .await
        .map_err(|e| ApiError::Database(e.to_string()))?;

    Ok(StatusCode::OK)
}

/// Send guest payment link via WhatsApp
async fn send_guest_payment_link(
    State(state): State<GuestState>,
    Path(token): Path<String>,
) -> Result<StatusCode, ApiError> {
    let parts: Vec<&str> = token.split('_').collect();
    if parts.len() < 2 {
        return Err(ApiError::BadRequest("Invalid token format".to_string()));
    }

    let invoice_id_str = parts[1];
    let invoice_id = Uuid::parse_str(invoice_id_str)
        .map_err(|_| ApiError::BadRequest("Invalid invoice ID".to_string()))?;

    // Get invoice
    let invoice = state
        .invoice_repo
        .get_invoice_by_id(invoice_id)
        .await
        .map_err(|_| ApiError::NotFound)?;

    // Get recent payment count (mock - would query database)
    let recent_payments_count = 0;

    // Send WhatsApp notification
    if let Some(phone) = invoice.client_phone.clone() {
        let payment_link = format!("https://yourapp.com/guest/pay/{}", token);

        let _ = state
            .notification_service
            .send_guest_notification(
                &invoice,
                Some(phone),
                payment_link,
                recent_payments_count,
            )
            .await;
    }

    Ok(StatusCode::OK)
}

/// Get discussion messages for guest (by token)
async fn get_discussion_messages_guest(
    State(state): State<GuestState>,
    Path(token): Path<String>,
) -> Result<Json<DiscussionResponseDto>, ApiError> {
    let parts: Vec<&str> = token.split('_').collect();
    if parts.len() < 2 {
        return Err(ApiError::BadRequest("Invalid token format".to_string()));
    }

    let invoice_id_str = parts[1];
    let invoice_id = Uuid::parse_str(invoice_id_str)
        .map_err(|_| ApiError::BadRequest("Invalid invoice ID".to_string()))?;

    let messages = state
        .invoice_service
        .get_discussion_messages_guest(invoice_id)
        .await
        .map_err(|e| ApiError::Database(e.to_string()))?;

    let dtos = messages.into_iter().map(|msg| DiscussionMessageDto {
        id: msg.id,
        invoice_id: msg.invoice_id,
        sender_type: msg.sender_type.to_string(),
        message: msg.message,
        created_at: msg.created_at,
    }).collect::<Vec<_>>();

    Ok(Json(DiscussionResponseDto { messages: dtos }))
}

/// Add discussion message as guest (buyer)
async fn add_discussion_message_guest(
    State(state): State<GuestState>,
    Path(token): Path<String>,
    Json(payload): Json<AddDiscussionMessageCommand>,
) -> Result<(StatusCode, Json<DiscussionMessageDto>), ApiError> {
    let parts: Vec<&str> = token.split('_').collect();
    if parts.len() < 2 {
        return Err(ApiError::BadRequest("Invalid token format".to_string()));
    }

    let invoice_id_str = parts[1];
    let invoice_id = Uuid::parse_str(invoice_id_str)
        .map_err(|_| ApiError::BadRequest("Invalid invoice ID".to_string()))?;

    let discussion = state
        .invoice_service
        .add_discussion_message_guest(invoice_id, payload.message)
        .await
        .map_err(|e| ApiError::Database(e.to_string()))?;

    let dto = DiscussionMessageDto {
        id: discussion.id,
        invoice_id: discussion.invoice_id,
        sender_type: discussion.sender_type.to_string(),
        message: discussion.message,
        created_at: discussion.created_at,
    };

    Ok((StatusCode::CREATED, Json(dto)))
}

pub fn create_guest_router(state: GuestState) -> Router {
    Router::new()
        .route("/invoice/{token}", get(get_invoice_by_token))
        .route("/pay/{token}", post(process_guest_payment))
        .route("/history", post(get_guest_payment_history))
        .route("/view/{token}", post(mark_invoice_viewed))
        .route("/send-link/{token}", post(send_guest_payment_link))
        .route("/discussion/{token}", get(get_discussion_messages_guest))
        .route("/discussion/{token}", post(add_discussion_message_guest))
        .with_state(state)
}
