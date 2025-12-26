use std::sync::Arc;
use uuid::Uuid;
use thiserror::Error;

use crate::domain::services::SettingsService;
use crate::domain::models::{BusinessAddress, TaxSettings, NotificationSettings, User};

#[derive(Debug, Error)]
pub enum SettingsError {
    #[error("User not found")]
    UserNotFound,
    #[error("Database error: {0}")]
    DatabaseError(String),
}

impl From<crate::domain::services::SettingsError> for SettingsError {
    fn from(err: crate::domain::services::SettingsError) -> Self {
        match err {
            crate::domain::services::SettingsError::UserNotFound => SettingsError::UserNotFound,
            crate::domain::services::SettingsError::DatabaseError(e) => SettingsError::DatabaseError(e),
        }
    }
}

// GetBusinessSettingsUseCase
#[derive(Clone)]
pub struct GetBusinessSettingsUseCase {
    settings_service: Arc<SettingsService>,
}

impl GetBusinessSettingsUseCase {
    pub fn new(settings_service: Arc<SettingsService>) -> Self {
        Self { settings_service }
    }

    pub async fn execute(&self, user_id: Uuid) -> Result<User, SettingsError> {
        Ok(self.settings_service.get_business_settings(user_id).await?)
    }
}

// UpdateBusinessSettingsUseCase
#[derive(Clone)]
pub struct UpdateBusinessSettingsUseCase {
    settings_service: Arc<SettingsService>,
}

impl UpdateBusinessSettingsUseCase {
    pub fn new(settings_service: Arc<SettingsService>) -> Self {
        Self { settings_service }
    }

    pub async fn execute(
        &self,
        user_id: Uuid,
        company_name: Option<String>,
        business_type: Option<String>,
        business_address: Option<BusinessAddress>,
        phone: Option<String>,
    ) -> Result<User, SettingsError> {
        Ok(self.settings_service.update_business_settings(
            user_id,
            company_name,
            business_type,
            business_address,
            phone,
        ).await?)
    }
}

// GetTaxSettingsUseCase
#[derive(Clone)]
pub struct GetTaxSettingsUseCase {
    settings_service: Arc<SettingsService>,
}

impl GetTaxSettingsUseCase {
    pub fn new(settings_service: Arc<SettingsService>) -> Self {
        Self { settings_service }
    }

    pub async fn execute(&self, user_id: Uuid) -> Result<User, SettingsError> {
        Ok(self.settings_service.get_tax_settings(user_id).await?)
    }
}

// UpdateTaxSettingsUseCase
#[derive(Clone)]
pub struct UpdateTaxSettingsUseCase {
    settings_service: Arc<SettingsService>,
}

impl UpdateTaxSettingsUseCase {
    pub fn new(settings_service: Arc<SettingsService>) -> Self {
        Self { settings_service }
    }

    pub async fn execute(
        &self,
        user_id: Uuid,
        tax_settings: TaxSettings,
    ) -> Result<User, SettingsError> {
        Ok(self.settings_service.update_tax_settings(user_id, tax_settings).await?)
    }
}

// GetNotificationSettingsUseCase
#[derive(Clone)]
pub struct GetNotificationSettingsUseCase {
    settings_service: Arc<SettingsService>,
}

impl GetNotificationSettingsUseCase {
    pub fn new(settings_service: Arc<SettingsService>) -> Self {
        Self { settings_service }
    }

    pub async fn execute(&self, user_id: Uuid) -> Result<User, SettingsError> {
        Ok(self.settings_service.get_notification_settings(user_id).await?)
    }
}

// UpdateNotificationSettingsUseCase
#[derive(Clone)]
pub struct UpdateNotificationSettingsUseCase {
    settings_service: Arc<SettingsService>,
}

impl UpdateNotificationSettingsUseCase {
    pub fn new(settings_service: Arc<SettingsService>) -> Self {
        Self { settings_service }
    }

    pub async fn execute(
        &self,
        user_id: Uuid,
        notification_settings: NotificationSettings,
    ) -> Result<User, SettingsError> {
        Ok(self.settings_service.update_notification_settings(user_id, notification_settings).await?)
    }
}

// GetInvoiceSettingsUseCase
#[derive(Clone)]
pub struct GetInvoiceSettingsUseCase {
    settings_service: Arc<SettingsService>,
}

impl GetInvoiceSettingsUseCase {
    pub fn new(settings_service: Arc<SettingsService>) -> Self {
        Self { settings_service }
    }

    pub async fn execute(&self, user_id: Uuid) -> Result<User, SettingsError> {
        Ok(self.settings_service.get_invoice_settings(user_id).await?)
    }
}

// UpdateInvoiceSettingsUseCase
#[derive(Clone)]
pub struct UpdateInvoiceSettingsUseCase {
    settings_service: Arc<SettingsService>,
}

impl UpdateInvoiceSettingsUseCase {
    pub fn new(settings_service: Arc<SettingsService>) -> Self {
        Self { settings_service }
    }

    pub async fn execute(
        &self,
        user_id: Uuid,
        template: String,
        logo_url: Option<String>,
        terms: String,
        notes: String,
    ) -> Result<User, SettingsError> {
        Ok(self.settings_service.update_invoice_settings(user_id, template, logo_url, terms, notes).await?)
    }
}
