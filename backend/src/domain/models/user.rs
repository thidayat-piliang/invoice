use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Type;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(type_name = "varchar")]
pub enum SubscriptionTier {
    Free,
    Pro,
    Business,
}

impl Default for SubscriptionTier {
    fn default() -> Self {
        SubscriptionTier::Free
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(type_name = "varchar")]
pub enum SubscriptionStatus {
    Active,
    Cancelled,
    Paused,
    Expired,
}

impl Default for SubscriptionStatus {
    fn default() -> Self {
        SubscriptionStatus::Active
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct BusinessAddress {
    pub street: String,
    pub city: String,
    pub state: String,
    pub zip_code: String,
    pub country: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSettings {
    pub email_payment_received: bool,
    pub email_invoice_paid: bool,
    pub email_payment_reminder: bool,
    pub push_payment_received: bool,
    pub push_overdue: bool,
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            email_payment_received: true,
            email_invoice_paid: true,
            email_payment_reminder: true,
            push_payment_received: true,
            push_overdue: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct TaxSettings {
    pub state_code: String,
    pub tax_rate: f64,
    pub tax_exempt: bool,
    pub tax_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InvoiceSettings {
    pub template: String,
    pub logo_url: Option<String>,
    pub terms: String,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct User {
    pub id: Uuid,

    #[validate(email)]
    pub email: String,

    pub phone: Option<String>,
    pub company_name: Option<String>,
    pub business_type: Option<String>,

    // Authentication
    pub password_hash: Option<String>,
    pub email_verified: bool,
    pub verification_token: Option<String>,
    pub reset_token: Option<String>,
    pub reset_token_expires: Option<DateTime<Utc>>,

    // Subscription
    pub subscription_tier: SubscriptionTier,
    pub subscription_status: SubscriptionStatus,
    pub stripe_customer_id: Option<String>,
    pub stripe_subscription_id: Option<String>,
    pub current_period_end: Option<DateTime<Utc>>,

    // Business Info
    pub business_address: Option<BusinessAddress>,
    pub tax_settings: Option<TaxSettings>,
    pub currency: String,

    // Settings
    pub notification_settings: NotificationSettings,
    pub invoice_settings: Option<InvoiceSettings>,

    // Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateUser {
    #[validate(email)]
    pub email: String,

    #[validate(length(min = 8))]
    pub password: String,

    pub phone: Option<String>,
    pub company_name: Option<String>,
    pub business_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateUser {
    pub phone: Option<String>,
    pub company_name: Option<String>,
    pub business_type: Option<String>,
    pub business_address: Option<BusinessAddress>,
    pub tax_settings: Option<TaxSettings>,
    pub notification_settings: Option<NotificationSettings>,
    pub invoice_settings: Option<InvoiceSettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUser {
    pub id: Uuid,
    pub email: String,
    pub subscription_tier: SubscriptionTier,
    pub company_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, validator::Validate)]
pub struct RegisterRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
    pub company_name: Option<String>,
    pub business_type: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResetPasswordRequest {
    pub token: String,
    pub new_password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub user: AuthUser,
}
