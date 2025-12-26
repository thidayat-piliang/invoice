use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Authentication required")]
    Unauthorized,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Resource not found")]
    NotFound,

    #[error("Forbidden")]
    Forbidden,

    #[error("Database error: {0}")]
    Database(String),

    #[error("Internal server error")]
    Internal,

    #[error("Rate limit exceeded")]
    RateLimit,

    #[error("Invalid input: {0}")]
    BadRequest(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message, code) = match self {
            ApiError::Validation(msg) => (StatusCode::BAD_REQUEST, msg, "VALIDATION_ERROR"),
            ApiError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".to_string(), "UNAUTHORIZED"),
            ApiError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "Invalid credentials".to_string(), "INVALID_CREDENTIALS"),
            ApiError::NotFound => (StatusCode::NOT_FOUND, "Not found".to_string(), "NOT_FOUND"),
            ApiError::Forbidden => (StatusCode::FORBIDDEN, "Forbidden".to_string(), "FORBIDDEN"),
            ApiError::Database(msg) => {
                tracing::error!("Database error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string(), "DATABASE_ERROR")
            }
            ApiError::Internal => {
                tracing::error!("Internal server error");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string(), "INTERNAL_ERROR")
            }
            ApiError::RateLimit => (StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded".to_string(), "RATE_LIMIT"),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg, "BAD_REQUEST"),
        };

        let body = json!({
            "error": {
                "code": code,
                "message": message,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }
        });

        (status, Json(body)).into_response()
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        ApiError::Database(err.to_string())
    }
}

impl From<validator::ValidationErrors> for ApiError {
    fn from(err: validator::ValidationErrors) -> Self {
        ApiError::Validation(err.to_string())
    }
}

impl From<crate::domain::services::AuthError> for ApiError {
    fn from(err: crate::domain::services::AuthError) -> Self {
        match err {
            crate::domain::services::AuthError::InvalidCredentials => ApiError::InvalidCredentials,
            crate::domain::services::AuthError::TokenExpired => ApiError::Unauthorized,
            crate::domain::services::AuthError::InvalidToken => ApiError::Unauthorized,
            crate::domain::services::AuthError::Validation(msg) => ApiError::Validation(msg),
            crate::domain::services::AuthError::DatabaseError(msg) => ApiError::Database(msg),
            crate::domain::services::AuthError::UserNotFound => ApiError::NotFound,
            crate::domain::services::AuthError::HashingFailed => ApiError::Internal,
        }
    }
}

impl From<crate::domain::services::InvoiceError> for ApiError {
    fn from(err: crate::domain::services::InvoiceError) -> Self {
        match err {
            crate::domain::services::InvoiceError::NotFound => ApiError::NotFound,
            crate::domain::services::InvoiceError::ClientNotFound => ApiError::BadRequest("Client not found".to_string()),
            crate::domain::services::InvoiceError::InvalidStatus(msg) => ApiError::BadRequest(msg),
            crate::domain::services::InvoiceError::DatabaseError(msg) => ApiError::Database(msg),
            crate::domain::services::InvoiceError::Validation(msg) => ApiError::Validation(msg),
            crate::domain::services::InvoiceError::PdfGenerationError(msg) => {
                tracing::error!("PDF generation error: {}", msg);
                ApiError::Internal
            }
            crate::domain::services::InvoiceError::EmailError(msg) => {
                tracing::error!("Email error: {}", msg);
                ApiError::Internal
            }
        }
    }
}

impl From<crate::application::use_cases::ReportError> for ApiError {
    fn from(err: crate::application::use_cases::ReportError) -> Self {
        match err {
            crate::application::use_cases::ReportError::DatabaseError(msg) => ApiError::Database(msg),
            crate::application::use_cases::ReportError::ExportError(msg) => {
                tracing::error!("Export error: {}", msg);
                ApiError::Internal
            }
        }
    }
}

impl From<crate::application::use_cases::SettingsError> for ApiError {
    fn from(err: crate::application::use_cases::SettingsError) -> Self {
        match err {
            crate::application::use_cases::SettingsError::UserNotFound => ApiError::NotFound,
            crate::application::use_cases::SettingsError::DatabaseError(msg) => ApiError::Database(msg),
        }
    }
}

impl From<crate::application::use_cases::ClientError> for ApiError {
    fn from(err: crate::application::use_cases::ClientError) -> Self {
        match err {
            crate::application::use_cases::ClientError::NotFound => ApiError::NotFound,
            crate::application::use_cases::ClientError::DatabaseError(msg) => ApiError::Database(msg),
        }
    }
}

impl From<crate::application::use_cases::PaymentError> for ApiError {
    fn from(err: crate::application::use_cases::PaymentError) -> Self {
        match err {
            crate::application::use_cases::PaymentError::NotFound => ApiError::NotFound,
            crate::application::use_cases::PaymentError::DatabaseError(msg) => ApiError::Database(msg),
        }
    }
}

impl From<crate::application::use_cases::ExpenseError> for ApiError {
    fn from(err: crate::application::use_cases::ExpenseError) -> Self {
        match err {
            crate::application::use_cases::ExpenseError::NotFound => ApiError::NotFound,
            crate::application::use_cases::ExpenseError::DatabaseError(msg) => ApiError::Database(msg),
        }
    }
}

impl From<crate::domain::services::TaxError> for ApiError {
    fn from(err: crate::domain::services::TaxError) -> Self {
        match err {
            crate::domain::services::TaxError::InvalidRate(msg) => ApiError::BadRequest(msg),
            crate::domain::services::TaxError::NotFound => ApiError::NotFound,
            crate::domain::services::TaxError::AlreadyExists => ApiError::BadRequest("Tax setting already exists".to_string()),
            crate::domain::services::TaxError::DefaultAlreadyExists => ApiError::BadRequest("Default tax already exists".to_string()),
            crate::domain::services::TaxError::DatabaseError(msg) => ApiError::Database(msg),
            crate::domain::services::TaxError::Validation(msg) => ApiError::Validation(msg),
        }
    }
}
