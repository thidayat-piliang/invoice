use std::sync::Arc;
use uuid::Uuid;
use chrono::NaiveDate;

use crate::infrastructure::repositories::ExpenseRepository;
use crate::domain::models::{Expense, ExpenseResponse, ExpenseStats, CreateExpense, UpdateExpense};

#[derive(Clone)]
pub struct ExpenseService {
    expense_repo: Arc<ExpenseRepository>,
}

impl ExpenseService {
    pub fn new(expense_repo: Arc<ExpenseRepository>) -> Self {
        Self { expense_repo }
    }

    pub async fn create_expense(
        &self,
        user_id: Uuid,
        create: CreateExpense,
    ) -> Result<Expense, sqlx::Error> {
        self.expense_repo.create(
            user_id,
            create.amount,
            "USD".to_string(),
            create.category,
            create.vendor,
            create.description,
            create.receipt_image_url,
            create.date_incurred,
            create.tax_deductible,
        ).await
    }

    pub async fn get_expense(
        &self,
        user_id: Uuid,
        expense_id: Uuid,
    ) -> Result<Option<Expense>, sqlx::Error> {
        self.expense_repo.find_by_id(user_id, expense_id).await
    }

    pub async fn list_expenses(
        &self,
        user_id: Uuid,
        category: Option<crate::domain::models::ExpenseCategory>,
        date_from: Option<NaiveDate>,
        date_to: Option<NaiveDate>,
        tax_deductible: Option<bool>,
        search: Option<String>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<ExpenseResponse>, sqlx::Error> {
        self.expense_repo.list(user_id, category, date_from, date_to, tax_deductible, search, limit, offset).await
    }

    pub async fn update_expense(
        &self,
        user_id: Uuid,
        expense_id: Uuid,
        update: UpdateExpense,
    ) -> Result<Expense, sqlx::Error> {
        self.expense_repo.update(
            user_id,
            expense_id,
            update.amount,
            update.category,
            update.vendor,
            update.description,
            update.receipt_image_url,
            update.date_incurred,
            update.tax_deductible,
        ).await
    }

    pub async fn delete_expense(&self, user_id: Uuid, expense_id: Uuid) -> Result<(), sqlx::Error> {
        self.expense_repo.delete(user_id, expense_id).await
    }

    pub async fn get_stats(&self, user_id: Uuid) -> Result<ExpenseStats, sqlx::Error> {
        self.expense_repo.get_stats(user_id).await
    }
}
