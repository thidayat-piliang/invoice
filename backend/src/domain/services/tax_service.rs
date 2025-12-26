use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TaxError {
    #[error("Invalid state code: {0}")]
    InvalidState(String),

    #[error("Tax calculation failed")]
    CalculationFailed,
}

pub struct TaxService {
    state_tax_rates: HashMap<String, f64>,
    local_tax_rates: HashMap<String, f64>,
}

impl TaxService {
    pub fn new() -> Self {
        let mut state_tax_rates = HashMap::new();

        // US State sales tax rates
        state_tax_rates.insert("CA".to_string(), 7.25); // California
        state_tax_rates.insert("NY".to_string(), 8.875); // New York
        state_tax_rates.insert("TX".to_string(), 6.25); // Texas
        state_tax_rates.insert("FL".to_string(), 6.00); // Florida
        state_tax_rates.insert("IL".to_string(), 6.25); // Illinois
        state_tax_rates.insert("PA".to_string(), 6.00); // Pennsylvania
        state_tax_rates.insert("OH".to_string(), 5.75); // Ohio
        state_tax_rates.insert("GA".to_string(), 4.00); // Georgia
        state_tax_rates.insert("NC".to_string(), 4.75); // North Carolina
        state_tax_rates.insert("MI".to_string(), 6.00); // Michigan

        Self {
            state_tax_rates,
            local_tax_rates: HashMap::new(),
        }
    }

    pub fn calculate_tax(
        &self,
        amount: f64,
        state_code: &str,
        city: Option<&str>,
        tax_exempt: bool,
    ) -> Result<f64, TaxError> {
        if tax_exempt {
            return Ok(0.0);
        }

        let state_rate = self.state_tax_rates
            .get(state_code)
            .ok_or_else(|| TaxError::InvalidState(state_code.to_string()))?;

        let local_rate = city.and_then(|c| self.local_tax_rates.get(c))
            .unwrap_or(&0.0);

        let total_rate = state_rate + local_rate;
        let tax_amount = amount * (total_rate / 100.0);

        Ok((tax_amount * 100.0).round() / 100.0)
    }

    pub fn validate_tax_id(&self, tax_id: &str, state: &str) -> bool {
        match state {
            "CA" => self.validate_ca_tax_id(tax_id),
            "NY" => self.validate_ny_tax_id(tax_id),
            _ => tax_id.len() >= 8 && tax_id.len() <= 12,
        }
    }

    fn validate_ca_tax_id(&self, tax_id: &str) -> bool {
        // California tax ID validation: CA + 8 digits
        tax_id.starts_with("CA") && tax_id.len() == 10
    }

    fn validate_ny_tax_id(&self, tax_id: &str) -> bool {
        // New York tax ID validation: NY + 10 digits
        tax_id.starts_with("NY") && tax_id.len() == 12
    }

    pub fn get_state_rate(&self, state_code: &str) -> Option<&f64> {
        self.state_tax_rates.get(state_code)
    }

    pub fn list_all_rates(&self) -> Vec<(String, f64)> {
        self.state_tax_rates
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect()
    }
}
