use async_trait::async_trait;
use uuid::Uuid;
use crate::domain::models::TaxSetting;

#[async_trait]
pub trait TaxRepository: Send + Sync {
    /// Create a new tax setting
    async fn create(&self, tax_setting: TaxSetting) -> Result<TaxSetting, Box<dyn std::error::Error + Send + Sync>>;

    /// Find all tax settings for an organization
    async fn find_by_organization(&self, organization_id: Uuid) -> Result<Vec<TaxSetting>, Box<dyn std::error::Error + Send + Sync>>;

    /// Find a specific tax setting by ID
    async fn find_by_id(&self, organization_id: Uuid, id: Uuid) -> Option<TaxSetting>;

    /// Find the default tax setting for an organization
    async fn find_default(&self, organization_id: Uuid) -> Result<Option<TaxSetting>, Box<dyn std::error::Error + Send + Sync>>;

    /// Update a tax setting
    async fn update(&self, tax_setting: TaxSetting) -> Result<TaxSetting, Box<dyn std::error::Error + Send + Sync>>;

    /// Delete a tax setting
    async fn delete(&self, organization_id: Uuid, id: Uuid) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// Unset all defaults for an organization
    async fn unset_all_defaults(&self, organization_id: Uuid) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}
