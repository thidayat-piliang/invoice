#![allow(dead_code)]

use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::{get, post, put},
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::api::error::ApiError;
use crate::api::middleware::AuthUser;
use crate::application::dto::auth_dto::*;
use crate::application::use_cases::*;

#[derive(Clone)]
struct AuthState {
    register_uc: Arc<RegisterUserUseCase>,
    login_uc: Arc<LoginUserUseCase>,
    refresh_token_uc: Arc<RefreshTokenUseCase>,
    forgot_password_uc: Arc<ForgotPasswordUseCase>,
    reset_password_uc: Arc<ResetPasswordUseCase>,
    verify_email_uc: Arc<VerifyEmailUseCase>,
    get_current_user_uc: Arc<GetCurrentUserUseCase>,
    update_profile_uc: Arc<UpdateProfileUseCase>,
}

pub fn create_router(
    register_uc: Arc<RegisterUserUseCase>,
    login_uc: Arc<LoginUserUseCase>,
    refresh_token_uc: Arc<RefreshTokenUseCase>,
    forgot_password_uc: Arc<ForgotPasswordUseCase>,
    reset_password_uc: Arc<ResetPasswordUseCase>,
    verify_email_uc: Arc<VerifyEmailUseCase>,
    get_current_user_uc: Arc<GetCurrentUserUseCase>,
    update_profile_uc: Arc<UpdateProfileUseCase>,
) -> Router {
    let state = AuthState {
        register_uc,
        login_uc,
        refresh_token_uc,
        forgot_password_uc,
        reset_password_uc,
        verify_email_uc,
        get_current_user_uc,
        update_profile_uc,
    };

    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/refresh", post(refresh_token))
        .route("/logout", post(logout))
        .route("/forgot-password", post(forgot_password))
        .route("/reset-password", post(reset_password))
        .route("/verify-email", get(verify_email))
        .route("/me", get(get_current_user))
        .route("/me", put(update_profile))
        .with_state(state)
}

async fn register(
    State(state): State<AuthState>,
    Json(payload): Json<RegisterUserCommand>,
) -> Result<(StatusCode, Json<AuthResultDto>), ApiError> {
    let response = state.register_uc.execute(payload).await?;
    Ok((StatusCode::CREATED, Json(response)))
}

async fn login(
    State(state): State<AuthState>,
    Json(payload): Json<LoginUserCommand>,
) -> Result<Json<AuthResultDto>, ApiError> {
    let response = state.login_uc.execute(payload).await?;
    Ok(Json(response))
}

async fn refresh_token(
    State(state): State<AuthState>,
    Json(payload): Json<RefreshTokenCommand>,
) -> Result<Json<AuthResultDto>, ApiError> {
    let response = state.refresh_token_uc.execute(payload).await?;
    Ok(Json(response))
}

async fn logout(
    _auth_user: AuthUser,
) -> Result<StatusCode, ApiError> {
    Ok(StatusCode::OK)
}

async fn forgot_password(
    state: State<AuthState>,
    Json(payload): Json<ForgotPasswordCommand>,
) -> Result<StatusCode, ApiError> {
    state.forgot_password_uc.execute(payload).await?;
    Ok(StatusCode::OK)
}

async fn reset_password(
    state: State<AuthState>,
    Json(payload): Json<ResetPasswordCommand>,
) -> Result<StatusCode, ApiError> {
    state.reset_password_uc.execute(payload).await?;
    Ok(StatusCode::OK)
}

async fn verify_email(
    state: State<AuthState>,
    Query(params): Query<VerifyEmailParams>,
) -> Result<Json<VerificationResultDto>, ApiError> {
    let result = state.verify_email_uc.execute(params.token).await?;
    Ok(Json(result))
}

async fn get_current_user(
    auth_user: AuthUser,
    state: State<AuthState>,
) -> Result<Json<UserDto>, ApiError> {
    let user = state.get_current_user_uc.execute(auth_user.user_id).await?;
    Ok(Json(user))
}

async fn update_profile(
    auth_user: AuthUser,
    state: State<AuthState>,
    Json(payload): Json<UpdateProfileCommand>,
) -> Result<Json<UserDto>, ApiError> {
    let user = state.update_profile_uc.execute(auth_user.user_id, payload).await?;
    Ok(Json(user))
}

#[derive(Deserialize)]
struct RefreshTokenRequest {
    refresh_token: String,
}

#[derive(Deserialize)]
struct VerifyEmailParams {
    token: String,
}
