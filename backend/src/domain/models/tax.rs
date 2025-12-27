#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use validator::Validate;

/// Tax Setting Model
/// Menyimpan tax rate manual per organization
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TaxSetting {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub label: String,           // "Sales Tax", "VAT", "GST"
    pub rate: f64,               // 0.00 - 1.00 (e.g. 0.07 for 7%)
    pub is_default: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create Tax Setting Request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateTaxSetting {
    #[validate(length(min = 1, max = 100, message = "Label is required"))]
    pub label: String,

    #[validate(range(min = 0.0, max = 1.0, message = "Rate must be between 0 and 1"))]
    pub rate: f64,

    pub is_default: Option<bool>,
}

/// Update Tax Setting Request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateTaxSetting {
    #[validate(length(min = 1, max = 100))]
    pub label: Option<String>,

    #[validate(range(min = 0.0, max = 1.0))]
    pub rate: Option<f64>,

    pub is_default: Option<bool>,
    pub is_active: Option<bool>,
}

/// Tax Calculation Result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxCalculation {
    pub subtotal: f64,
    pub tax_rate: f64,
    pub tax_amount: f64,
    pub total: f64,
    pub tax_label: String,
}

/// Tax Summary for Report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxSummary {
    pub period_start: String,
    pub period_end: String,
    pub subtotal: f64,           // Taxable amount
    pub tax_collected: f64,      // Total tax
    pub total: f64,              // Subtotal + Tax
    pub tax_breakdown: Vec<TaxBreakdownItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxBreakdownItem {
    pub label: String,
    pub rate: f64,
    pub taxable_amount: f64,
    pub tax_amount: f64,
}

/// Tax ID Validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxIdValidation {
    pub tax_id: String,
    pub is_valid: bool,
    pub normalized: String,
}

impl TaxSetting {
    /// Calculate tax for a given amount
    pub fn calculate_tax(&self, amount: f64) -> TaxCalculation {
        let tax_amount = (amount * self.rate * 100.0).round() / 100.0; // Round to 2 decimals
        let total = ((amount + tax_amount) * 100.0).round() / 100.0; // Round to 2 decimals

        TaxCalculation {
            subtotal: amount,
            tax_rate: self.rate,
            tax_amount,
            total,
            tax_label: self.label.clone(),
        }
    }
}

/// Validation for tax rate
pub fn validate_tax_rate(rate: f64) -> bool {
    rate >= 0.0 && rate <= 1.0
}

/// Validation for tax ID (EIN format: XX-XXXXXXX)
pub fn validate_tax_id(tax_id: &str) -> bool {
    if tax_id.is_empty() {
        return true; // Empty is valid (optional)
    }
    if tax_id.len() > 32 {
        return false;
    }
    // EIN format: 2 digits, hyphen, 7 digits (XX-XXXXXXX)
    let parts: Vec<&str> = tax_id.split('-').collect();
    if parts.len() != 2 {
        return false;
    }
    let first = parts[0];
    let second = parts[1];
    first.len() == 2 && first.chars().all(|c| c.is_ascii_digit()) &&
    second.len() == 7 && second.chars().all(|c| c.is_ascii_digit())
}

/// Normalize tax ID (uppercase, remove special chars except alphanumeric)
pub fn normalize_tax_id(tax_id: &str) -> String {
    tax_id.to_uppercase()
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect()
}
