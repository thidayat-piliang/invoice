use uuid::Uuid;
use thiserror::Error;

use crate::domain::models::{User, UpdateUser, BusinessAddress, TaxSettings, NotificationSettings, InvoiceSettings};
use crate::infrastructure::repositories::UserRepository;

#[derive(Debug, Error)]
pub enum SettingsError {
    #[error("User not found")]
    UserNotFound,
    #[error("Database error: {0}")]
    DatabaseError(String),
}

impl From<sqlx::Error> for SettingsError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => SettingsError::UserNotFound,
            _ => SettingsError::DatabaseError(err.to_string()),
        }
    }
}

#[derive(Clone)]
pub struct SettingsService {
    user_repo: UserRepository,
}

impl SettingsService {
    pub fn new(user_repo: UserRepository) -> Self {
        Self { user_repo }
    }

    pub async fn get_business_settings(&self, user_id: Uuid) -> Result<User, SettingsError> {
        let user = self.user_repo.find_by_id(user_id).await?;
        user.ok_or(SettingsError::UserNotFound)
    }

    pub async fn update_business_settings(
        &self,
        user_id: Uuid,
        company_name: Option<String>,
        business_type: Option<String>,
        business_address: Option<BusinessAddress>,
        phone: Option<String>,
    ) -> Result<User, SettingsError> {
        let update = UpdateUser {
            phone,
            company_name,
            business_type,
            business_address,
            tax_settings: None,
            notification_settings: None,
            invoice_settings: None,
        };

        Ok(self.user_repo.update(user_id, update).await?)
    }

    pub async fn get_tax_settings(&self, user_id: Uuid) -> Result<User, SettingsError> {
        let user = self.user_repo.find_by_id(user_id).await?;
        user.ok_or(SettingsError::UserNotFound)
    }

    pub async fn update_tax_settings(
        &self,
        user_id: Uuid,
        tax_settings: TaxSettings,
    ) -> Result<User, SettingsError> {
        let update = UpdateUser {
            phone: None,
            company_name: None,
            business_type: None,
            business_address: None,
            tax_settings: Some(tax_settings),
            notification_settings: None,
            invoice_settings: None,
        };

        Ok(self.user_repo.update(user_id, update).await?)
    }

    pub async fn get_notification_settings(&self, user_id: Uuid) -> Result<User, SettingsError> {
        let user = self.user_repo.find_by_id(user_id).await?;
        user.ok_or(SettingsError::UserNotFound)
    }

    pub async fn update_notification_settings(
        &self,
        user_id: Uuid,
        notification_settings: NotificationSettings,
    ) -> Result<User, SettingsError> {
        let update = UpdateUser {
            phone: None,
            company_name: None,
            business_type: None,
            business_address: None,
            tax_settings: None,
            notification_settings: Some(notification_settings),
            invoice_settings: None,
        };

        Ok(self.user_repo.update(user_id, update).await?)
    }

    pub async fn get_invoice_settings(&self, user_id: Uuid) -> Result<User, SettingsError> {
        let user = self.user_repo.find_by_id(user_id).await?;
        user.ok_or(SettingsError::UserNotFound)
    }

    pub async fn update_invoice_settings(
        &self,
        user_id: Uuid,
        template: String,
        logo_url: Option<String>,
        terms: String,
        notes: String,
    ) -> Result<User, SettingsError> {
        let update = UpdateUser {
            phone: None,
            company_name: None,
            business_type: None,
            business_address: None,
            tax_settings: None,
            notification_settings: None,
            invoice_settings: Some(InvoiceSettings {
                template,
                logo_url,
                terms,
                notes,
            }),
        };

        Ok(self.user_repo.update(user_id, update).await?)
    }
}
