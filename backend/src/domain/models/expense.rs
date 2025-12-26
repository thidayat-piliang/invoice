use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Type;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(type_name = "varchar")]
pub enum ExpenseCategory {
    Supplies,
    Travel,
    Equipment,
    Software,
    Marketing,
    Utilities,
    Other,
}

impl std::fmt::Display for ExpenseCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExpenseCategory::Supplies => write!(f, "supplies"),
            ExpenseCategory::Travel => write!(f, "travel"),
            ExpenseCategory::Equipment => write!(f, "equipment"),
            ExpenseCategory::Software => write!(f, "software"),
            ExpenseCategory::Marketing => write!(f, "marketing"),
            ExpenseCategory::Utilities => write!(f, "utilities"),
            ExpenseCategory::Other => write!(f, "other"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Expense {
    pub id: Uuid,
    pub user_id: Uuid,

    #[validate(range(min = 0.01))]
    pub amount: f64,

    pub currency: String,
    pub category: ExpenseCategory,

    pub vendor: Option<String>,
    pub description: Option<String>,
    pub receipt_image_url: Option<String>,

    pub date_incurred: NaiveDate,
    pub tax_deductible: bool,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateExpense {
    #[validate(range(min = 0.01))]
    pub amount: f64,

    pub category: ExpenseCategory,
    pub vendor: Option<String>,
    pub description: Option<String>,
    pub receipt_image_url: Option<String>,
    pub date_incurred: NaiveDate,
    pub tax_deductible: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateExpense {
    pub amount: Option<f64>,
    pub category: Option<ExpenseCategory>,
    pub vendor: Option<String>,
    pub description: Option<String>,
    pub receipt_image_url: Option<String>,
    pub date_incurred: Option<NaiveDate>,
    pub tax_deductible: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpenseListFilter {
    pub category: Option<ExpenseCategory>,
    pub date_from: Option<NaiveDate>,
    pub date_to: Option<NaiveDate>,
    pub tax_deductible: Option<bool>,
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpenseResponse {
    pub id: Uuid,
    pub amount: f64,
    pub currency: String,
    pub category: ExpenseCategory,
    pub vendor: Option<String>,
    pub description: Option<String>,
    pub date_incurred: NaiveDate,
    pub tax_deductible: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpenseStats {
    pub total_expenses: f64,
    pub tax_deductible: f64,
    pub by_category: std::collections::HashMap<String, f64>,
    pub monthly_average: f64,
}
