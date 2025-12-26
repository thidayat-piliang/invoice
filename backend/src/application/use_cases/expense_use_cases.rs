use std::sync::Arc;
use uuid::Uuid;
use chrono::NaiveDate;
use thiserror::Error;

use crate::domain::services::ExpenseService;
use crate::domain::models::{Expense, ExpenseResponse, ExpenseStats, CreateExpense, UpdateExpense, ExpenseCategory};

#[derive(Debug, Error)]
pub enum ExpenseError {
    #[error("Expense not found")]
    NotFound,
    #[error("Database error: {0}")]
    DatabaseError(String),
}

impl From<sqlx::Error> for ExpenseError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => ExpenseError::NotFound,
            _ => ExpenseError::DatabaseError(err.to_string()),
        }
    }
}

// CreateExpenseUseCase
#[derive(Clone)]
pub struct CreateExpenseUseCase {
    expense_service: Arc<ExpenseService>,
}

impl CreateExpenseUseCase {
    pub fn new(expense_service: Arc<ExpenseService>) -> Self {
        Self { expense_service }
    }

    pub async fn execute(&self, user_id: Uuid, create: CreateExpense) -> Result<Expense, ExpenseError> {
        Ok(self.expense_service.create_expense(user_id, create).await?)
    }
}

// GetExpenseUseCase
#[derive(Clone)]
pub struct GetExpenseUseCase {
    expense_service: Arc<ExpenseService>,
}

impl GetExpenseUseCase {
    pub fn new(expense_service: Arc<ExpenseService>) -> Self {
        Self { expense_service }
    }

    pub async fn execute(&self, user_id: Uuid, expense_id: Uuid) -> Result<Expense, ExpenseError> {
        let expense = self.expense_service.get_expense(user_id, expense_id).await?;
        expense.ok_or(ExpenseError::NotFound)
    }
}

// ListExpensesUseCase
#[derive(Clone)]
pub struct ListExpensesUseCase {
    expense_service: Arc<ExpenseService>,
}

impl ListExpensesUseCase {
    pub fn new(expense_service: Arc<ExpenseService>) -> Self {
        Self { expense_service }
    }

    pub async fn execute(
        &self,
        user_id: Uuid,
        category: Option<ExpenseCategory>,
        date_from: Option<NaiveDate>,
        date_to: Option<NaiveDate>,
        tax_deductible: Option<bool>,
        search: Option<String>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<ExpenseResponse>, ExpenseError> {
        Ok(self.expense_service.list_expenses(
            user_id, category, date_from, date_to, tax_deductible, search, limit, offset
        ).await?)
    }
}

// UpdateExpenseUseCase
#[derive(Clone)]
pub struct UpdateExpenseUseCase {
    expense_service: Arc<ExpenseService>,
}

impl UpdateExpenseUseCase {
    pub fn new(expense_service: Arc<ExpenseService>) -> Self {
        Self { expense_service }
    }

    pub async fn execute(
        &self,
        user_id: Uuid,
        expense_id: Uuid,
        update: UpdateExpense,
    ) -> Result<Expense, ExpenseError> {
        Ok(self.expense_service.update_expense(user_id, expense_id, update).await?)
    }
}

// DeleteExpenseUseCase
#[derive(Clone)]
pub struct DeleteExpenseUseCase {
    expense_service: Arc<ExpenseService>,
}

impl DeleteExpenseUseCase {
    pub fn new(expense_service: Arc<ExpenseService>) -> Self {
        Self { expense_service }
    }

    pub async fn execute(&self, user_id: Uuid, expense_id: Uuid) -> Result<(), ExpenseError> {
        Ok(self.expense_service.delete_expense(user_id, expense_id).await?)
    }
}

// GetExpenseStatsUseCase
#[derive(Clone)]
pub struct GetExpenseStatsUseCase {
    expense_service: Arc<ExpenseService>,
}

impl GetExpenseStatsUseCase {
    pub fn new(expense_service: Arc<ExpenseService>) -> Self {
        Self { expense_service }
    }

    pub async fn execute(&self, user_id: Uuid) -> Result<ExpenseStats, ExpenseError> {
        Ok(self.expense_service.get_stats(user_id).await?)
    }
}
