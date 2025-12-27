use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::models::{SubscriptionTier, SubscriptionStatus};

// Input DTOs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterUserCommand {
    pub email: String,
    pub password: String,
    pub phone: Option<String>,
    pub company_name: Option<String>,
    pub business_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginUserCommand {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenCommand {
    pub refresh_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgotPasswordCommand {
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResetPasswordCommand {
    pub token: String,
    pub new_password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProfileCommand {
    pub phone: Option<String>,
    pub company_name: Option<String>,
    pub business_type: Option<String>,
    pub business_address: Option<serde_json::Value>,
    pub tax_settings: Option<serde_json::Value>,
    pub notification_settings: Option<serde_json::Value>,
}

// Output DTOs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticatedUserDto {
    pub id: Uuid,
    pub email: String,
    pub subscription_tier: SubscriptionTier,
    pub company_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResultDto {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub user: AuthenticatedUserDto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDto {
    pub id: Uuid,
    pub email: String,
    pub phone: Option<String>,
    pub company_name: Option<String>,
    pub business_type: Option<String>,
    pub email_verified: bool,
    pub subscription_tier: SubscriptionTier,
    pub subscription_status: SubscriptionStatus,
    pub business_address: Option<serde_json::Value>,
    pub tax_settings: Option<serde_json::Value>,
    pub currency: String,
    pub notification_settings: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub last_login_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResultDto {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct PasswordResetResultDto {
    pub success: bool,
    pub message: String,
}
