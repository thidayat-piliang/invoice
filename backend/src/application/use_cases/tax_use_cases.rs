use crate::domain::services::{TaxService, TaxError};
use crate::domain::models::{TaxSetting, CreateTaxSetting, UpdateTaxSetting, TaxCalculation, TaxSummary};
use std::sync::Arc;
use uuid::Uuid;

/// Create Tax Setting Use Case
pub struct CreateTaxSettingUseCase {
    tax_service: Arc<TaxService>,
}

impl CreateTaxSettingUseCase {
    pub fn new(tax_service: Arc<TaxService>) -> Self {
        Self { tax_service }
    }

    pub async fn execute(
        &self,
        organization_id: Uuid,
        input: CreateTaxSetting,
    ) -> Result<TaxSetting, TaxError> {
        self.tax_service.create_tax_setting(organization_id, input).await
    }
}

/// Get Organization Tax Settings Use Case
pub struct GetOrganizationTaxSettingsUseCase {
    tax_service: Arc<TaxService>,
}

impl GetOrganizationTaxSettingsUseCase {
    pub fn new(tax_service: Arc<TaxService>) -> Self {
        Self { tax_service }
    }

    pub async fn execute(&self, organization_id: Uuid) -> Result<Vec<TaxSetting>, TaxError> {
        self.tax_service.get_tax_settings(organization_id).await
    }
}

/// Get Default Tax Use Case
pub struct GetDefaultTaxUseCase {
    tax_service: Arc<TaxService>,
}

impl GetDefaultTaxUseCase {
    pub fn new(tax_service: Arc<TaxService>) -> Self {
        Self { tax_service }
    }

    pub async fn execute(&self, organization_id: Uuid) -> Result<Option<TaxSetting>, TaxError> {
        self.tax_service.get_default_tax(organization_id).await
    }
}

/// Update Tax Setting Use Case
pub struct UpdateTaxSettingUseCase {
    tax_service: Arc<TaxService>,
}

impl UpdateTaxSettingUseCase {
    pub fn new(tax_service: Arc<TaxService>) -> Self {
        Self { tax_service }
    }

    pub async fn execute(
        &self,
        organization_id: Uuid,
        id: Uuid,
        input: UpdateTaxSetting,
    ) -> Result<TaxSetting, TaxError> {
        self.tax_service.update_tax_setting(organization_id, id, input).await
    }
}

/// Delete Tax Setting Use Case
pub struct DeleteTaxSettingUseCase {
    tax_service: Arc<TaxService>,
}

impl DeleteTaxSettingUseCase {
    pub fn new(tax_service: Arc<TaxService>) -> Self {
        Self { tax_service }
    }

    pub async fn execute(&self, organization_id: Uuid, id: Uuid) -> Result<(), TaxError> {
        self.tax_service.delete_tax_setting(organization_id, id).await
    }
}

/// Calculate Tax Use Case
pub struct CalculateTaxUseCase {
    tax_service: Arc<TaxService>,
}

impl CalculateTaxUseCase {
    pub fn new(tax_service: Arc<TaxService>) -> Self {
        Self { tax_service }
    }

    pub async fn execute(&self, organization_id: Uuid, amount: f64) -> Result<TaxCalculation, TaxError> {
        self.tax_service.calculate_tax(organization_id, amount).await
    }
}

/// Get Tax Summary Use Case
pub struct GetTaxSummaryUseCase {
    tax_service: Arc<TaxService>,
}

impl GetTaxSummaryUseCase {
    pub fn new(tax_service: Arc<TaxService>) -> Self {
        Self { tax_service }
    }

    pub async fn execute(
        &self,
        organization_id: Uuid,
        invoices: Vec<crate::domain::models::Invoice>,
    ) -> Result<TaxSummary, TaxError> {
        self.tax_service.get_tax_summary(organization_id, invoices).await
    }
}

/// Validate Tax ID Use Case
pub struct ValidateTaxIdUseCase {
    tax_service: Arc<TaxService>,
}

impl ValidateTaxIdUseCase {
    pub fn new(tax_service: Arc<TaxService>) -> Self {
        Self { tax_service }
    }

    pub fn execute(&self, tax_id: &str) -> bool {
        self.tax_service.validate_tax_id(tax_id)
    }
}
