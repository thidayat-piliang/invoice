use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::IntoResponse,
    RequestPartsExt,
};
use axum_extra::{
    headers::{Authorization, authorization::Bearer},
    TypedHeader,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

use crate::domain::services::AuthService;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct _AuthClaims {
    pub sub: String,
    pub email: String,
    pub exp: usize,
    pub iat: usize,
    pub tier: String,
}

#[derive(Debug, Error)]
pub enum AuthExtractorError {
    #[error("Missing authorization header")]
    MissingAuth,

    #[error("Invalid token")]
    InvalidToken,

    #[error("Token expired")]
    TokenExpired,

    #[error("Unauthorized")]
    Unauthorized,
}

impl IntoResponse for AuthExtractorError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            AuthExtractorError::MissingAuth => (StatusCode::UNAUTHORIZED, "Missing authorization header"),
            AuthExtractorError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid token"),
            AuthExtractorError::TokenExpired => (StatusCode::UNAUTHORIZED, "Token expired"),
            AuthExtractorError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized"),
        };

        let body = serde_json::json!({
            "error": {
                "code": "AUTH_ERROR",
                "message": message,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }
        });

        (status, axum::Json(body)).into_response()
    }
}

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub _email: String,
    pub _tier: String,
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AuthExtractorError;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        // Extract header first (needs mutable borrow)
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthExtractorError::MissingAuth)?;

        // Get AuthService from request extensions (added by middleware)
        let auth_service = parts
            .extensions
            .get::<Arc<AuthService>>()
            .ok_or(AuthExtractorError::Unauthorized)?;

        let claims = auth_service
            .verify_token(bearer.token())
            .map_err(|_| AuthExtractorError::InvalidToken)?;

        let user_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| AuthExtractorError::InvalidToken)?;

        Ok(AuthUser {
            user_id,
            _email: claims.email,
            _tier: claims.tier,
        })
    }
}
