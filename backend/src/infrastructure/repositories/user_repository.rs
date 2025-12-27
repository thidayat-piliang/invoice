use sqlx::{PgPool, Postgres, QueryBuilder};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::domain::models::{User, UpdateUser, SubscriptionTier, SubscriptionStatus, InvoiceSettings};

#[derive(Clone)]
pub struct UserRepository {
    db: PgPool,
}

impl UserRepository {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn create(
        &self,
        email: String,
        password_hash: String,
        phone: Option<String>,
        company_name: Option<String>,
        business_type: Option<String>,
        verification_token: String,
    ) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as::<_, UserRow>(
            r#"
            INSERT INTO users (
                id, email, phone, company_name, business_type, password_hash,
                email_verified, verification_token, subscription_tier, subscription_status,
                currency, notification_settings, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(&email)
        .bind(&phone)
        .bind(&company_name)
        .bind(&business_type)
        .bind(&password_hash)
        .bind(false) // email_verified
        .bind(&verification_token) // verification_token
        .bind("free") // subscription_tier
        .bind("active") // subscription_status
        .bind("USD")
        .bind(serde_json::json!({
            "email_payment_received": true,
            "email_invoice_paid": true,
            "email_payment_reminder": true,
            "push_payment_received": true,
            "push_overdue": true
        }))
        .bind(Utc::now())
        .bind(Utc::now())
        .fetch_one(&self.db)
        .await?;

        Ok(user.to_user())
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error> {
        let user = sqlx::query_as::<_, UserRow>(
            "SELECT * FROM users WHERE email = $1"
        )
        .bind(email)
        .fetch_optional(&self.db)
        .await?;

        Ok(user.map(|u| u.to_user()))
    }

    pub async fn find_by_id(&self, user_id: Uuid) -> Result<Option<User>, sqlx::Error> {
        let user = sqlx::query_as::<_, UserRow>(
            "SELECT * FROM users WHERE id = $1"
        )
        .bind(user_id)
        .fetch_optional(&self.db)
        .await?;

        Ok(user.map(|u| u.to_user()))
    }

    pub async fn update(
        &self,
        user_id: Uuid,
        update: UpdateUser,
    ) -> Result<User, sqlx::Error> {
        let mut query_builder = QueryBuilder::<Postgres>::new(
            "UPDATE users SET updated_at = "
        );
        query_builder.push_bind(Utc::now());

        if let Some(ref phone) = update.phone {
            query_builder.push(", phone = ");
            query_builder.push_bind(phone);
        }

        if let Some(ref company_name) = update.company_name {
            query_builder.push(", company_name = ");
            query_builder.push_bind(company_name);
        }

        if let Some(ref business_type) = update.business_type {
            query_builder.push(", business_type = ");
            query_builder.push_bind(business_type);
        }

        if let Some(ref business_address) = update.business_address {
            query_builder.push(", business_address = ");
            let json = serde_json::to_value(business_address).unwrap_or(serde_json::Value::Null);
            query_builder.push_bind(json);
        }

        if let Some(ref tax_settings) = update.tax_settings {
            query_builder.push(", tax_settings = ");
            let json = serde_json::to_value(tax_settings).unwrap_or(serde_json::Value::Null);
            query_builder.push_bind(json);
        }

        if let Some(ref notification_settings) = update.notification_settings {
            query_builder.push(", notification_settings = ");
            let json = serde_json::to_value(notification_settings).unwrap_or(serde_json::Value::Null);
            query_builder.push_bind(json);
        }

        if let Some(ref invoice_settings) = update.invoice_settings {
            query_builder.push(", invoice_settings = ");
            let json = serde_json::to_value(invoice_settings).unwrap_or(serde_json::Value::Null);
            query_builder.push_bind(json);
        }

        query_builder.push(" WHERE id = ");
        query_builder.push_bind(user_id);
        query_builder.push(" RETURNING *");

        let user = query_builder
            .build_query_as::<UserRow>()
            .fetch_one(&self.db)
            .await?;

        Ok(user.to_user())
    }

    pub async fn update_password_hash(&self, user_id: Uuid, password_hash: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE users SET password_hash = $1, updated_at = $2 WHERE id = $3"
        )
        .bind(password_hash)
        .bind(Utc::now())
        .bind(user_id)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    pub async fn verify_email(&self, user_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE users SET email_verified = true, updated_at = $1 WHERE id = $2"
        )
        .bind(Utc::now())
        .bind(user_id)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    pub async fn update_reset_token(
        &self,
        user_id: Uuid,
        token: &str,
        expires: DateTime<Utc>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE users SET reset_token = $1, reset_token_expires = $2, updated_at = $3 WHERE id = $4"
        )
        .bind(token)
        .bind(expires)
        .bind(Utc::now())
        .bind(user_id)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    pub async fn find_by_reset_token(&self, token: &str) -> Result<Option<User>, sqlx::Error> {
        let user = sqlx::query_as::<_, UserRow>(
            "SELECT * FROM users WHERE reset_token = $1 AND reset_token_expires > NOW()"
        )
        .bind(token)
        .fetch_optional(&self.db)
        .await?;

        Ok(user.map(|u| u.to_user()))
    }

    pub async fn find_by_verification_token(&self, token: &str) -> Result<Option<User>, sqlx::Error> {
        let user = sqlx::query_as::<_, UserRow>(
            "SELECT * FROM users WHERE verification_token = $1"
        )
        .bind(token)
        .fetch_optional(&self.db)
        .await?;

        Ok(user.map(|u| u.to_user()))
    }

    pub async fn update_last_login(&self, user_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE users SET last_login_at = $1, updated_at = $2 WHERE id = $3"
        )
        .bind(Utc::now())
        .bind(Utc::now())
        .bind(user_id)
        .execute(&self.db)
        .await?;

        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct UserRow {
    id: Uuid,
    email: String,
    phone: Option<String>,
    company_name: Option<String>,
    business_type: Option<String>,
    password_hash: Option<String>,
    email_verified: bool,
    verification_token: Option<String>,
    reset_token: Option<String>,
    reset_token_expires: Option<DateTime<Utc>>,
    subscription_tier: String,
    subscription_status: String,
    stripe_customer_id: Option<String>,
    stripe_subscription_id: Option<String>,
    current_period_end: Option<DateTime<Utc>>,
    business_address: Option<serde_json::Value>,
    tax_settings: Option<serde_json::Value>,
    currency: String,
    notification_settings: serde_json::Value,
    invoice_settings: Option<serde_json::Value>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    last_login_at: Option<DateTime<Utc>>,
}

impl UserRow {
    fn to_user(self) -> User {
        let subscription_tier = match self.subscription_tier.as_str() {
            "pro" => SubscriptionTier::Pro,
            "business" => SubscriptionTier::Business,
            _ => SubscriptionTier::Free,
        };

        let subscription_status = match self.subscription_status.as_str() {
            "active" => SubscriptionStatus::Active,
            "canceled" => SubscriptionStatus::Cancelled,
            "expired" => SubscriptionStatus::Expired,
            "paused" => SubscriptionStatus::Paused,
            _ => SubscriptionStatus::Active,
        };

        // Deserialize JSON fields to domain types
        let business_address: Option<crate::domain::models::BusinessAddress> = self.business_address
            .and_then(|v| serde_json::from_value(v).ok());

        let tax_settings: Option<crate::domain::models::TaxSettings> = self.tax_settings
            .and_then(|v| serde_json::from_value(v).ok());

        let notification_settings: crate::domain::models::NotificationSettings =
            serde_json::from_value(self.notification_settings).unwrap_or_default();

        let invoice_settings: Option<InvoiceSettings> = self.invoice_settings
            .and_then(|v| serde_json::from_value(v).ok());

        User {
            id: self.id,
            email: self.email,
            phone: self.phone,
            company_name: self.company_name,
            business_type: self.business_type,
            password_hash: self.password_hash,
            email_verified: self.email_verified,
            verification_token: self.verification_token,
            reset_token: self.reset_token,
            reset_token_expires: self.reset_token_expires,
            subscription_tier,
            subscription_status,
            stripe_customer_id: self.stripe_customer_id,
            stripe_subscription_id: self.stripe_subscription_id,
            current_period_end: self.current_period_end,
            business_address,
            tax_settings,
            currency: self.currency,
            notification_settings,
            invoice_settings,
            created_at: self.created_at,
            updated_at: self.updated_at,
            last_login_at: self.last_login_at,
        }
    }
}
