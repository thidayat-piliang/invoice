use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post, put, delete},
    Json, Router,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::api::error::ApiError;
use crate::api::middleware::AuthUser;
use crate::domain::models::{CreateExpense, UpdateExpense, ExpenseListFilter};
use crate::application::use_cases::{
    CreateExpenseUseCase, GetExpenseUseCase, ListExpensesUseCase,
    UpdateExpenseUseCase, DeleteExpenseUseCase, GetExpenseStatsUseCase,
};

#[derive(Clone)]
struct ExpenseState {
    create_expense_uc: Arc<CreateExpenseUseCase>,
    get_expense_uc: Arc<GetExpenseUseCase>,
    list_expenses_uc: Arc<ListExpensesUseCase>,
    update_expense_uc: Arc<UpdateExpenseUseCase>,
    delete_expense_uc: Arc<DeleteExpenseUseCase>,
    get_expense_stats_uc: Arc<GetExpenseStatsUseCase>,
}

pub fn create_router(
    create_expense_uc: Arc<CreateExpenseUseCase>,
    get_expense_uc: Arc<GetExpenseUseCase>,
    list_expenses_uc: Arc<ListExpensesUseCase>,
    update_expense_uc: Arc<UpdateExpenseUseCase>,
    delete_expense_uc: Arc<DeleteExpenseUseCase>,
    get_expense_stats_uc: Arc<GetExpenseStatsUseCase>,
) -> Router {
    let state = ExpenseState {
        create_expense_uc,
        get_expense_uc,
        list_expenses_uc,
        update_expense_uc,
        delete_expense_uc,
        get_expense_stats_uc,
    };

    Router::new()
        .route("/", get(list_expenses))
        .route("/", post(create_expense))
        .route("/{id}", get(get_expense))
        .route("/{id}", put(update_expense))
        .route("/{id}", delete(delete_expense))
        .route("/stats", get(get_expense_stats))
        .with_state(state)
}

async fn list_expenses(
    auth_user: AuthUser,
    State(state): State<ExpenseState>,
    Query(filter): Query<ExpenseListFilter>,
) -> Result<Json<Vec<crate::domain::models::ExpenseResponse>>, ApiError> {
    let expenses = state.list_expenses_uc.execute(
        auth_user.user_id,
        filter.category,
        filter.date_from,
        filter.date_to,
        filter.tax_deductible,
        filter.search,
        filter.limit,
        filter.offset,
    ).await?;
    Ok(Json(expenses))
}

async fn create_expense(
    auth_user: AuthUser,
    State(state): State<ExpenseState>,
    Json(payload): Json<CreateExpense>,
) -> Result<(StatusCode, Json<crate::domain::models::Expense>), ApiError> {
    let expense = state.create_expense_uc.execute(auth_user.user_id, payload).await?;
    Ok((StatusCode::CREATED, Json(expense)))
}

async fn get_expense(
    auth_user: AuthUser,
    State(state): State<ExpenseState>,
    Path(expense_id): Path<Uuid>,
) -> Result<Json<crate::domain::models::Expense>, ApiError> {
    let expense = state.get_expense_uc.execute(auth_user.user_id, expense_id).await?;
    Ok(Json(expense))
}

async fn update_expense(
    auth_user: AuthUser,
    State(state): State<ExpenseState>,
    Path(expense_id): Path<Uuid>,
    Json(payload): Json<UpdateExpense>,
) -> Result<Json<crate::domain::models::Expense>, ApiError> {
    let expense = state.update_expense_uc.execute(auth_user.user_id, expense_id, payload).await?;
    Ok(Json(expense))
}

async fn delete_expense(
    auth_user: AuthUser,
    State(state): State<ExpenseState>,
    Path(expense_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.delete_expense_uc.execute(auth_user.user_id, expense_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_expense_stats(
    auth_user: AuthUser,
    State(state): State<ExpenseState>,
) -> Result<Json<crate::domain::models::ExpenseStats>, ApiError> {
    let stats = state.get_expense_stats_uc.execute(auth_user.user_id).await?;
    Ok(Json(stats))
}
