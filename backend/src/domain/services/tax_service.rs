use crate::domain::models::{
    TaxSetting, CreateTaxSetting, UpdateTaxSetting, TaxCalculation, TaxSummary,
    TaxBreakdownItem, validate_tax_rate, validate_tax_id
};
use crate::domain::repositories::tax_repository::TaxRepository;
use std::sync::Arc;
use uuid::Uuid;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TaxError {
    #[error("Tax rate invalid: {0}")]
    InvalidRate(String),

    #[error("Tax setting not found")]
    NotFound,

    #[error("Tax setting already exists")]
    AlreadyExists,

    #[error("Default tax already exists for this organization")]
    DefaultAlreadyExists,

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Validation error: {0}")]
    Validation(String),
}

impl From<Box<dyn std::error::Error + Send + Sync>> for TaxError {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        TaxError::DatabaseError(err.to_string())
    }
}

/// Tax Service
/// Handles tax calculation and management
///
/// NOTE: This is informational only. Flashbill does not calculate,
/// verify, or file taxes on your behalf.
pub struct TaxService {
    repository: Arc<dyn TaxRepository + Send + Sync>,
}

impl TaxService {
    pub fn new(repository: Arc<dyn TaxRepository + Send + Sync>) -> Self {
        Self { repository }
    }

    /// Create a new tax setting
    pub async fn create_tax_setting(
        &self,
        organization_id: Uuid,
        input: CreateTaxSetting,
    ) -> Result<TaxSetting, TaxError> {
        // Validate rate
        if !validate_tax_rate(input.rate) {
            return Err(TaxError::InvalidRate("Rate must be between 0 and 1".to_string()));
        }

        // Check if default already exists
        if input.is_default.unwrap_or(false) {
            if let Some(existing) = self.repository.find_default(organization_id).await? {
                // If updating existing default, allow it
                if existing.label != input.label || existing.rate != input.rate {
                    return Err(TaxError::DefaultAlreadyExists);
                }
            }
        }

        let tax_setting = TaxSetting {
            id: Uuid::new_v4(),
            organization_id,
            label: input.label.clone(),
            rate: input.rate,
            is_default: input.is_default.unwrap_or(false),
            is_active: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        self.repository.create(tax_setting).await
            .map_err(|e| TaxError::DatabaseError(e.to_string()))
    }

    /// Get all tax settings for an organization
    pub async fn get_tax_settings(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<TaxSetting>, TaxError> {
        self.repository.find_by_organization(organization_id).await
            .map_err(|e| TaxError::DatabaseError(e.to_string()))
    }

    /// Get a specific tax setting
    pub async fn get_tax_setting(
        &self,
        organization_id: Uuid,
        id: Uuid,
    ) -> Result<TaxSetting, TaxError> {
        self.repository.find_by_id(organization_id, id).await
            .ok_or(TaxError::NotFound)
    }

    /// Get default tax setting
    pub async fn get_default_tax(
        &self,
        organization_id: Uuid,
    ) -> Result<Option<TaxSetting>, TaxError> {
        self.repository.find_default(organization_id).await
            .map_err(|e| TaxError::DatabaseError(e.to_string()))
    }

    /// Update a tax setting
    pub async fn update_tax_setting(
        &self,
        organization_id: Uuid,
        id: Uuid,
        input: UpdateTaxSetting,
    ) -> Result<TaxSetting, TaxError> {
        let mut tax_setting = self.get_tax_setting(organization_id, id).await?;

        if let Some(label) = input.label {
            tax_setting.label = label;
        }

        if let Some(rate) = input.rate {
            if !validate_tax_rate(rate) {
                return Err(TaxError::InvalidRate("Rate must be between 0 and 1".to_string()));
            }
            tax_setting.rate = rate;
        }

        if let Some(is_default) = input.is_default {
            if is_default {
                // Unset other defaults
                self.repository.unset_all_defaults(organization_id).await
                    .map_err(|e| TaxError::DatabaseError(e.to_string()))?;
            }
            tax_setting.is_default = is_default;
        }

        if let Some(is_active) = input.is_active {
            tax_setting.is_active = is_active;
        }

        tax_setting.updated_at = chrono::Utc::now();

        self.repository.update(tax_setting).await
            .map_err(|e| TaxError::DatabaseError(e.to_string()))
    }

    /// Delete a tax setting
    pub async fn delete_tax_setting(
        &self,
        organization_id: Uuid,
        id: Uuid,
    ) -> Result<(), TaxError> {
        let tax_setting = self.get_tax_setting(organization_id, id).await?;

        // Don't allow deleting if it's the only default
        if tax_setting.is_default {
            let others = self.repository.find_by_organization(organization_id).await
                .map_err(|e| TaxError::DatabaseError(e.to_string()))?;

            if others.len() == 1 {
                return Err(TaxError::Validation("Cannot delete the only default tax setting".to_string()));
            }
        }

        self.repository.delete(organization_id, id).await
            .map_err(|e| TaxError::DatabaseError(e.to_string()))
    }

    /// Calculate tax for an amount using default tax
    pub async fn calculate_tax(
        &self,
        organization_id: Uuid,
        amount: f64,
    ) -> Result<TaxCalculation, TaxError> {
        let default_tax = self.get_default_tax(organization_id).await?;

        match default_tax {
            Some(tax) => Ok(tax.calculate_tax(amount)),
            None => Ok(TaxCalculation {
                subtotal: amount,
                tax_rate: 0.0,
                tax_amount: 0.0,
                total: amount,
                tax_label: "No Tax".to_string(),
            }),
        }
    }

    /// Calculate tax for a line item
    pub async fn _calculate_line_tax(
        &self,
        organization_id: Uuid,
        quantity: f64,
        unit_price: f64,
        custom_tax_rate: Option<f64>,
    ) -> Result<TaxCalculation, TaxError> {
        let subtotal = quantity * unit_price;

        let tax_rate = match custom_tax_rate {
            Some(rate) => {
                if !validate_tax_rate(rate) {
                    return Err(TaxError::InvalidRate("Custom tax rate invalid".to_string()));
                }
                rate
            }
            None => {
                let default_tax = self.get_default_tax(organization_id).await?;
                match default_tax {
                    Some(tax) => tax.rate,
                    None => 0.0,
                }
            }
        };

        // Calculate with 2 decimal rounding
        let tax_amount = (subtotal * tax_rate * 100.0).round() / 100.0;
        let total = ((subtotal + tax_amount) * 100.0).round() / 100.0;

        let tax_label = match custom_tax_rate {
            Some(_) => "Custom Tax".to_string(),
            None => {
                let default_tax = self.get_default_tax(organization_id).await?;
                match default_tax {
                    Some(tax) => tax.label.clone(),
                    None => "No Tax".to_string(),
                }
            }
        };

        Ok(TaxCalculation {
            subtotal,
            tax_rate,
            tax_amount,
            total,
            tax_label,
        })
    }

    /// Get tax summary for a period
    pub async fn get_tax_summary(
        &self,
        organization_id: Uuid,
        invoices: Vec<crate::domain::models::Invoice>,
    ) -> Result<TaxSummary, TaxError> {
        let mut subtotal = 0.0;
        let mut tax_collected = 0.0;
        let mut total = 0.0;

        let mut breakdown_map = std::collections::HashMap::new();

        for invoice in invoices {
            subtotal += invoice.subtotal;
            tax_collected += invoice.tax_amount;
            total += invoice.total_amount;

            // Group by tax label
            if let Some(tax_label) = &invoice.tax_label {
                let entry = breakdown_map.entry(tax_label.clone())
                    .or_insert((0.0, 0.0));
                entry.0 += invoice.subtotal;
                entry.1 += invoice.tax_amount;
            }
        }

        let tax_breakdown = breakdown_map
            .into_iter()
            .map(|(label, (taxable, tax))| TaxBreakdownItem {
                label,
                rate: if taxable > 0.0 { tax / taxable } else { 0.0 },
                taxable_amount: taxable,
                tax_amount: tax,
            })
            .collect();

        Ok(TaxSummary {
            period_start: "Start".to_string(),  // Will be populated by caller
            period_end: "End".to_string(),
            subtotal,
            tax_collected,
            total,
            tax_breakdown,
        })
    }

    /// Validate tax ID
    pub fn validate_tax_id(&self, tax_id: &str) -> bool {
        validate_tax_id(tax_id)
    }

    /// Normalize tax ID
    pub fn _normalize_tax_id(&self, tax_id: &str) -> String {
        crate::domain::models::normalize_tax_id(tax_id)
    }
}
