use axum::{
    extract::State,
    http::StatusCode,
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::api::error::ApiError;
use crate::api::middleware::AuthUser;
use crate::domain::services::payment_gateway_service::{CreatePaymentIntent, PaymentGatewayService, RefundRequest};

#[derive(Clone)]
pub struct PayPalState {
    payment_gateway_service: Arc<PaymentGatewayService>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePayPalOrderRequest {
    pub amount: f64,
    pub currency: Option<String>,
    pub invoice_id: Option<Uuid>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreatePayPalOrderResponse {
    pub order_id: String,
    pub amount: f64,
    pub currency: String,
    pub status: String,
    pub payment_method: String,
    pub checkout_url: Option<String>,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct RefundPayPalRequest {
    pub payment_id: String,
    pub amount: Option<f64>,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RefundPayPalResponse {
    pub refund_id: String,
    pub amount: f64,
    pub status: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct PayPalStatusResponse {
    pub configured: bool,
    pub available_gateways: Vec<String>,
    pub client_id: Option<String>,
}

/// Create PayPal router with all payment gateway endpoints
pub fn create_paypal_router(payment_gateway_service: Arc<PaymentGatewayService>) -> Router {
    let state = PayPalState {
        payment_gateway_service,
    };

    Router::new()
        .route("/create-order", post(create_paypal_order))
        .route("/refund", post(refund_paypal_payment))
        .route("/status", post(get_paypal_status))
        .with_state(state)
}

/// Create a PayPal order for checkout
/// This creates an order that can be used with PayPal's JavaScript SDK
async fn create_paypal_order(
    _auth_user: AuthUser,
    State(state): State<PayPalState>,
    Json(payload): Json<CreatePayPalOrderRequest>,
) -> Result<(StatusCode, Json<CreatePayPalOrderResponse>), ApiError> {
    if payload.amount <= 0.0 {
        return Err(ApiError::BadRequest("Amount must be greater than 0".to_string()));
    }

    let intent = CreatePaymentIntent {
        amount: payload.amount,
        currency: payload.currency.unwrap_or_else(|| "USD".to_string()),
        description: payload.description,
        metadata: payload.invoice_id.map(|id| {
            let mut map = std::collections::HashMap::new();
            map.insert("invoice_id".to_string(), id.to_string());
            map
        }),
    };

    let payment_intent = state
        .payment_gateway_service
        .create_paypal_order(intent)
        .await
        .map_err(|_e| ApiError::Internal)?;

    // In production, you would also generate a PayPal checkout URL here
    // For now, return the order ID for client-side integration
    Ok((StatusCode::CREATED, Json(CreatePayPalOrderResponse {
        order_id: payment_intent.id.clone(),
        amount: payment_intent.amount,
        currency: payment_intent.currency,
        status: payment_intent.status,
        payment_method: payment_intent.payment_method,
        checkout_url: Some(format!("https://www.paypal.com/checkoutnow?token={}", payment_intent.id)),
        message: "PayPal order created successfully. Use the order_id with PayPal SDK or redirect to checkout_url".to_string(),
    })))
}

/// Refund a PayPal payment
async fn refund_paypal_payment(
    _auth_user: AuthUser,
    State(state): State<PayPalState>,
    Json(payload): Json<RefundPayPalRequest>,
) -> Result<Json<RefundPayPalResponse>, ApiError> {
    let refund_request = RefundRequest {
        payment_intent_id: payload.payment_id,
        amount: payload.amount,
        reason: payload.reason,
    };

    let refund = state
        .payment_gateway_service
        .refund_paypal_payment(refund_request)
        .await
        .map_err(|_e| ApiError::Internal)?;

    Ok(Json(RefundPayPalResponse {
        refund_id: refund.id,
        amount: refund.amount,
        status: refund.status,
        message: "PayPal refund processed successfully".to_string(),
    }))
}

/// Check PayPal gateway status and configuration
async fn get_paypal_status(
    _auth_user: AuthUser,
    State(state): State<PayPalState>,
) -> Result<Json<PayPalStatusResponse>, ApiError> {
    let configured = state.payment_gateway_service.is_paypal_configured();
    let available_gateways = state.payment_gateway_service.get_available_gateways();

    // Get client ID if configured (for display purposes)
    let client_id = if configured {
        // In a real implementation, you might store this differently
        Some("configured".to_string())
    } else {
        None
    };

    Ok(Json(PayPalStatusResponse {
        configured,
        available_gateways,
        client_id,
    }))
}
