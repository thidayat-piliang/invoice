use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{Duration, Utc};
use jsonwebtokens::{encode, Algorithm, AlgorithmID, Verifier};
use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;
use uuid::Uuid;
use validator::Validate;

use crate::domain::models::{RegisterRequest, LoginRequest, ForgotPasswordRequest, ResetPasswordRequest, AuthResponse, User, UpdateUser};
use crate::domain::services::EmailService;
use crate::infrastructure::repositories::UserRepository;
use std::sync::Arc;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Token expired")]
    TokenExpired,

    #[error("Invalid token")]
    InvalidToken,

    #[error("Password hashing failed")]
    HashingFailed,

    #[error("User not found")]
    UserNotFound,

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Database error: {0}")]
    DatabaseError(String),
}

impl From<sqlx::Error> for AuthError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => AuthError::UserNotFound,
            _ => AuthError::DatabaseError(err.to_string()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user_id
    pub email: String,
    pub exp: usize,
    pub iat: usize,
    pub tier: String,
}

pub struct AuthService {
    user_repo: UserRepository,
    email_service: Arc<EmailService>,
    jwt_secret: String,
    access_token_expiry: i64, // in minutes
    refresh_token_expiry: i64, // in days
}

impl AuthService {
    pub fn new(user_repo: UserRepository, email_service: Arc<EmailService>, jwt_secret: String) -> Self {
        Self {
            user_repo,
            email_service,
            jwt_secret,
            access_token_expiry: 60 * 24, // 24 hours
            refresh_token_expiry: 7, // 7 days
        }
    }

    pub fn hash_password(&self, password: &str) -> Result<String, AuthError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|_| AuthError::HashingFailed)
    }

    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool, AuthError> {
        let parsed_hash = PasswordHash::new(hash).map_err(|_| AuthError::HashingFailed)?;
        let argon2 = Argon2::default();

        argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .map(|_| true)
            .map_err(|_| AuthError::InvalidCredentials.into())
    }

    pub fn generate_access_token(
        &self,
        user_id: Uuid,
        email: &str,
        tier: &str,
    ) -> Result<String, AuthError> {
        let alg = Algorithm::new_hmac(AlgorithmID::HS256, self.jwt_secret.as_bytes())
            .map_err(|_| AuthError::HashingFailed)?;

        let header = json!({ "alg": alg.name() });
        let claims = json!({
            "sub": user_id.to_string(),
            "email": email,
            "exp": Utc::now().timestamp() + (self.access_token_expiry * 60),
            "iat": Utc::now().timestamp(),
            "tier": tier,
        });

        encode(&header, &claims, &alg).map_err(|_| AuthError::InvalidToken)
    }

    pub fn generate_refresh_token(&self, user_id: Uuid) -> Result<String, AuthError> {
        let alg = Algorithm::new_hmac(AlgorithmID::HS256, self.jwt_secret.as_bytes())
            .map_err(|_| AuthError::HashingFailed)?;

        let header = json!({ "alg": alg.name() });
        let claims = json!({
            "sub": user_id.to_string(),
            "email": "",
            "exp": Utc::now().timestamp() + (self.refresh_token_expiry * 24 * 60 * 60),
            "iat": Utc::now().timestamp(),
            "tier": "",
        });

        encode(&header, &claims, &alg).map_err(|_| AuthError::InvalidToken)
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, AuthError> {
        let alg = Algorithm::new_hmac(AlgorithmID::HS256, self.jwt_secret.as_bytes())
            .map_err(|_| AuthError::HashingFailed)?;

        let verifier = Verifier::create()
            .build()
            .map_err(|_| AuthError::InvalidToken)?;

        let claims_value = verifier
            .verify(token, &alg)
            .map_err(|_| AuthError::InvalidToken)?;

        // Convert Value to Claims
        let claims: Claims = serde_json::from_value(claims_value)
            .map_err(|_| AuthError::InvalidToken)?;

        if claims.exp < Utc::now().timestamp() as usize {
            return Err(AuthError::TokenExpired);
        }

        Ok(claims)
    }

    pub fn generate_verification_token(&self) -> String {
        Uuid::new_v4().to_string()
    }

    pub fn generate_reset_token(&self) -> (String, chrono::DateTime<Utc>) {
        let token = Uuid::new_v4().to_string();
        let expires = Utc::now() + Duration::hours(1);
        (token, expires)
    }

    pub fn create_token_hash(&self, token: &str) -> Result<String, AuthError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        argon2
            .hash_password(token.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|_| AuthError::HashingFailed)
    }

    pub fn verify_token_hash(&self, token: &str, hash: &str) -> Result<bool, AuthError> {
        let parsed_hash = PasswordHash::new(hash).map_err(|_| AuthError::HashingFailed)?;
        let argon2 = Argon2::default();

        argon2
            .verify_password(token.as_bytes(), &parsed_hash)
            .map(|_| true)
            .map_err(|_| AuthError::InvalidToken.into())
    }

    // High-level business logic methods (moved from routes)

    pub async fn register(&self, payload: RegisterRequest) -> Result<AuthResponse, AuthError> {
        // Validate
        payload.validate().map_err(|e| AuthError::Validation(e.to_string()))?;

        // Check if user already exists
        let existing_user = self.user_repo.find_by_email(&payload.email).await?;
        if existing_user.is_some() {
            return Err(AuthError::Validation("Email already registered".to_string()));
        }

        // Hash password
        let password_hash = self.hash_password(&payload.password)?;

        // Generate verification token
        let verification_token = self.generate_verification_token();

        // Clone email and company_name for email sending
        let email = payload.email.clone();
        let company_name = payload.company_name.clone();

        // Create user
        let user = self.user_repo.create(
            email.clone(),
            password_hash,
            payload.phone,
            payload.company_name,
            payload.business_type,
            verification_token.clone(),
        ).await?;

        // Send verification email (non-blocking, ignore errors)
        let _ = self.email_service.send_verification_email(
            &email,
            &company_name.unwrap_or_else(|| "Customer".to_string()),
            &verification_token,
        );

        // Generate tokens for immediate login
        let tier_str = match user.subscription_tier {
            crate::domain::models::SubscriptionTier::Free => "free",
            crate::domain::models::SubscriptionTier::Pro => "pro",
            crate::domain::models::SubscriptionTier::Business => "business",
        };
        let access_token = self.generate_access_token(
            user.id,
            &user.email,
            tier_str
        )?;
        let refresh_token = self.generate_refresh_token(user.id)?;

        Ok(AuthResponse {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: self.access_token_expiry * 60,
            user: crate::domain::models::AuthUser {
                id: user.id,
                email: user.email,
                subscription_tier: user.subscription_tier,
                company_name: user.company_name,
            },
        })
    }

    pub async fn login(&self, payload: LoginRequest) -> Result<AuthResponse, AuthError> {
        // Fetch user from DB
        let user = self.user_repo.find_by_email(&payload.email)
            .await?
            .ok_or(AuthError::InvalidCredentials)?;

        // Verify password
        let password_hash = user.password_hash.as_ref()
            .ok_or(AuthError::InvalidCredentials)?;

        if !self.verify_password(&payload.password, password_hash)? {
            return Err(AuthError::InvalidCredentials);
        }

        // Update last login
        self.user_repo.update_last_login(user.id).await?;

        // Generate tokens
        let tier_str = match user.subscription_tier {
            crate::domain::models::SubscriptionTier::Free => "free",
            crate::domain::models::SubscriptionTier::Pro => "pro",
            crate::domain::models::SubscriptionTier::Business => "business",
        };
        let access_token = self.generate_access_token(
            user.id,
            &user.email,
            tier_str
        )?;
        let refresh_token = self.generate_refresh_token(user.id)?;

        Ok(AuthResponse {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: 24 * 60 * 60,
            user: crate::domain::models::AuthUser {
                id: user.id,
                email: user.email,
                subscription_tier: user.subscription_tier,
                company_name: user.company_name,
            },
        })
    }

    pub async fn refresh_token(&self, refresh_token: String) -> Result<AuthResponse, AuthError> {
        let claims = self.verify_token(&refresh_token)?;
        let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AuthError::InvalidToken)?;

        // Fetch user to get current tier
        let user = self.user_repo.find_by_id(user_id)
            .await?
            .ok_or(AuthError::UserNotFound)?;

        let access_token = self.generate_access_token(
            user_id,
            &claims.email,
            &claims.tier,
        )?;

        Ok(AuthResponse {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: 24 * 60 * 60,
            user: crate::domain::models::AuthUser {
                id: user_id,
                email: claims.email.clone(),
                subscription_tier: user.subscription_tier,
                company_name: user.company_name,
            },
        })
    }

    pub async fn forgot_password(&self, payload: ForgotPasswordRequest) -> Result<(), AuthError> {
        // Fetch user
        let user = self.user_repo.find_by_email(&payload.email)
            .await?
            .ok_or(AuthError::UserNotFound)?;

        // Generate reset token
        let (token, expires) = self.generate_reset_token();

        // Store token in DB
        self.user_repo.update_reset_token(user.id, &token, expires).await?;

        // Send password reset email
        let company_name = user.company_name.clone().unwrap_or_else(|| "Customer".to_string());
        self.email_service.send_password_reset(
            &payload.email,
            &company_name,
            &token,
        ).map_err(|e| AuthError::DatabaseError(format!("Email send failed: {}", e)))?;

        Ok(())
    }

    pub async fn reset_password(&self, payload: ResetPasswordRequest) -> Result<(), AuthError> {
        // Find user by token
        let user = self.user_repo.find_by_reset_token(&payload.token)
            .await?
            .ok_or(AuthError::InvalidToken)?;

        // Hash new password
        let password_hash = self.hash_password(&payload.new_password)?;

        // Update password and clear reset token
        self.user_repo.update_password_hash(user.id, &password_hash).await?;

        // Clear the reset token
        let future_expiration = Utc::now() - Duration::hours(1); // Expired in the past
        self.user_repo.update_reset_token(user.id, "", future_expiration).await?;

        Ok(())
    }

    pub async fn verify_email(&self, token: String) -> Result<(), AuthError> {
        // Find user by verification token
        let user = self.user_repo.find_by_verification_token(&token)
            .await?
            .ok_or(AuthError::InvalidToken)?;

        // Mark email as verified
        self.user_repo.verify_email(user.id).await?;

        // Send welcome email (optional - could be a different template)
        // For now, we'll skip this to avoid email spam during testing
        // In production, you might send a "Welcome to FlashBill" email here

        Ok(())
    }

    pub async fn get_user(&self, user_id: Uuid) -> Result<User, AuthError> {
        let user = self.user_repo.find_by_id(user_id)
            .await?
            .ok_or(AuthError::UserNotFound)?;
        Ok(user)
    }

    pub async fn update_user(&self, user_id: Uuid, payload: UpdateUser) -> Result<User, AuthError> {
        // Verify user exists
        let _ = self.get_user(user_id).await?;

        // Update user
        let updated_user = self.user_repo.update(user_id, payload).await?;
        Ok(updated_user)
    }
}
