use std::sync::Arc;
use uuid::Uuid;

use crate::application::dto::auth_dto::*;
use crate::domain::models::{RegisterRequest, LoginRequest, ForgotPasswordRequest, ResetPasswordRequest};
use crate::domain::services::{AuthService, AuthError};

/// Use case: Register new user
pub struct RegisterUserUseCase {
    auth_service: Arc<AuthService>,
}

impl RegisterUserUseCase {
    pub fn new(auth_service: Arc<AuthService>) -> Self {
        Self { auth_service }
    }

    pub async fn execute(&self, command: RegisterUserCommand) -> Result<AuthResultDto, AuthError> {
        let request = RegisterRequest {
            email: command.email,
            password: command.password,
            phone: command.phone,
            company_name: command.company_name,
            business_type: command.business_type,
        };

        let response = self.auth_service.register(request).await?;

        Ok(AuthResultDto {
            access_token: response.access_token,
            refresh_token: response.refresh_token,
            token_type: response.token_type,
            expires_in: response.expires_in,
            user: AuthenticatedUserDto {
                id: response.user.id,
                email: response.user.email,
                subscription_tier: response.user.subscription_tier,
                company_name: response.user.company_name,
            },
        })
    }
}

/// Use case: Login user
pub struct LoginUserUseCase {
    auth_service: Arc<AuthService>,
}

impl LoginUserUseCase {
    pub fn new(auth_service: Arc<AuthService>) -> Self {
        Self { auth_service }
    }

    pub async fn execute(&self, command: LoginUserCommand) -> Result<AuthResultDto, AuthError> {
        let request = LoginRequest {
            email: command.email,
            password: command.password,
        };

        let response = self.auth_service.login(request).await?;

        Ok(AuthResultDto {
            access_token: response.access_token,
            refresh_token: response.refresh_token,
            token_type: response.token_type,
            expires_in: response.expires_in,
            user: AuthenticatedUserDto {
                id: response.user.id,
                email: response.user.email,
                subscription_tier: response.user.subscription_tier,
                company_name: response.user.company_name,
            },
        })
    }
}

/// Use case: Refresh token
pub struct RefreshTokenUseCase {
    auth_service: Arc<AuthService>,
}

impl RefreshTokenUseCase {
    pub fn new(auth_service: Arc<AuthService>) -> Self {
        Self { auth_service }
    }

    pub async fn execute(&self, command: RefreshTokenCommand) -> Result<AuthResultDto, AuthError> {
        let response = self.auth_service.refresh_token(command.refresh_token).await?;

        Ok(AuthResultDto {
            access_token: response.access_token,
            refresh_token: response.refresh_token,
            token_type: response.token_type,
            expires_in: response.expires_in,
            user: AuthenticatedUserDto {
                id: response.user.id,
                email: response.user.email,
                subscription_tier: response.user.subscription_tier,
                company_name: response.user.company_name,
            },
        })
    }
}

/// Use case: Forgot password
pub struct ForgotPasswordUseCase {
    auth_service: Arc<AuthService>,
}

impl ForgotPasswordUseCase {
    pub fn new(auth_service: Arc<AuthService>) -> Self {
        Self { auth_service }
    }

    pub async fn execute(&self, command: ForgotPasswordCommand) -> Result<(), AuthError> {
        let request = ForgotPasswordRequest {
            email: command.email,
        };

        self.auth_service.forgot_password(request).await
    }
}

/// Use case: Reset password
pub struct ResetPasswordUseCase {
    auth_service: Arc<AuthService>,
}

impl ResetPasswordUseCase {
    pub fn new(auth_service: Arc<AuthService>) -> Self {
        Self { auth_service }
    }

    pub async fn execute(&self, command: ResetPasswordCommand) -> Result<(), AuthError> {
        let request = ResetPasswordRequest {
            token: command.token,
            new_password: command.new_password,
        };

        self.auth_service.reset_password(request).await
    }
}

/// Use case: Verify email
pub struct VerifyEmailUseCase {
    auth_service: Arc<AuthService>,
}

impl VerifyEmailUseCase {
    pub fn new(auth_service: Arc<AuthService>) -> Self {
        Self { auth_service }
    }

    pub async fn execute(&self, token: String) -> Result<VerificationResultDto, AuthError> {
        self.auth_service.verify_email(token).await?;

        Ok(VerificationResultDto {
            success: true,
            message: "Email verified successfully".to_string(),
        })
    }
}

/// Use case: Get current user
pub struct GetCurrentUserUseCase {
    auth_service: Arc<AuthService>,
}

impl GetCurrentUserUseCase {
    pub fn new(auth_service: Arc<AuthService>) -> Self {
        Self { auth_service }
    }

    pub async fn execute(&self, user_id: Uuid) -> Result<UserDto, AuthError> {
        let user = self.auth_service.get_user(user_id).await?;

        Ok(UserDto {
            id: user.id,
            email: user.email,
            phone: user.phone,
            company_name: user.company_name,
            business_type: user.business_type,
            email_verified: user.email_verified,
            subscription_tier: user.subscription_tier,
            subscription_status: user.subscription_status,
            business_address: user.business_address.and_then(|v| serde_json::to_value(v).ok()),
            tax_settings: user.tax_settings.and_then(|v| serde_json::to_value(v).ok()),
            currency: user.currency,
            notification_settings: serde_json::to_value(user.notification_settings).unwrap_or_default(),
            created_at: user.created_at,
            updated_at: user.updated_at,
            last_login_at: user.last_login_at,
        })
    }
}

/// Use case: Update profile
pub struct UpdateProfileUseCase {
    auth_service: Arc<AuthService>,
}

impl UpdateProfileUseCase {
    pub fn new(auth_service: Arc<AuthService>) -> Self {
        Self { auth_service }
    }

    pub async fn execute(&self, user_id: Uuid, command: UpdateProfileCommand) -> Result<UserDto, AuthError> {
        // Convert JSON values to domain types
        let business_address = command.business_address
            .and_then(|v| serde_json::from_value(v).ok());
        let tax_settings = command.tax_settings
            .and_then(|v| serde_json::from_value(v).ok());
        let notification_settings = command.notification_settings
            .and_then(|v| serde_json::from_value(v).ok());

        let update = crate::domain::models::UpdateUser {
            phone: command.phone,
            company_name: command.company_name,
            business_type: command.business_type,
            business_address,
            tax_settings,
            notification_settings,
            invoice_settings: None,
        };

        let user = self.auth_service.update_user(user_id, update).await?;

        Ok(UserDto {
            id: user.id,
            email: user.email,
            phone: user.phone,
            company_name: user.company_name,
            business_type: user.business_type,
            email_verified: user.email_verified,
            subscription_tier: user.subscription_tier,
            subscription_status: user.subscription_status,
            business_address: user.business_address.and_then(|v| serde_json::to_value(v).ok()),
            tax_settings: user.tax_settings.and_then(|v| serde_json::to_value(v).ok()),
            currency: user.currency,
            notification_settings: serde_json::to_value(user.notification_settings).unwrap_or_default(),
            created_at: user.created_at,
            updated_at: user.updated_at,
            last_login_at: user.last_login_at,
        })
    }
}
