use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post, put, delete},
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::api::error::ApiError;
use crate::api::middleware::AuthUser;
use crate::domain::models::{CreateClient, UpdateClient, ClientListFilter};
use crate::application::use_cases::{
    CreateClientUseCase, GetClientUseCase, ListClientsUseCase,
    UpdateClientUseCase, DeleteClientUseCase, GetClientInvoicesUseCase,
    GetClientStatsUseCase,
};

#[derive(Clone)]
struct ClientState {
    create_client_uc: Arc<CreateClientUseCase>,
    get_client_uc: Arc<GetClientUseCase>,
    list_clients_uc: Arc<ListClientsUseCase>,
    update_client_uc: Arc<UpdateClientUseCase>,
    delete_client_uc: Arc<DeleteClientUseCase>,
    get_client_invoices_uc: Arc<GetClientInvoicesUseCase>,
    get_client_stats_uc: Arc<GetClientStatsUseCase>,
}

pub fn create_router(
    create_client_uc: Arc<CreateClientUseCase>,
    get_client_uc: Arc<GetClientUseCase>,
    list_clients_uc: Arc<ListClientsUseCase>,
    update_client_uc: Arc<UpdateClientUseCase>,
    delete_client_uc: Arc<DeleteClientUseCase>,
    get_client_invoices_uc: Arc<GetClientInvoicesUseCase>,
    get_client_stats_uc: Arc<GetClientStatsUseCase>,
) -> Router {
    let state = ClientState {
        create_client_uc,
        get_client_uc,
        list_clients_uc,
        update_client_uc,
        delete_client_uc,
        get_client_invoices_uc,
        get_client_stats_uc,
    };

    Router::new()
        .route("/", get(list_clients))
        .route("/", post(create_client))
        .route("/{id}", get(get_client))
        .route("/{id}", put(update_client))
        .route("/{id}", delete(delete_client))
        .route("/{id}/invoices", get(get_client_invoices))
        .route("/stats", get(get_client_stats))
        .with_state(state)
}

async fn list_clients(
    auth_user: AuthUser,
    State(state): State<ClientState>,
    Query(filter): Query<ClientListFilter>,
) -> Result<Json<Vec<crate::domain::models::ClientResponse>>, ApiError> {
    let clients = state.list_clients_uc.execute(
        auth_user.user_id,
        filter.search,
        filter.limit,
        filter.offset,
    ).await?;
    Ok(Json(clients))
}

async fn create_client(
    auth_user: AuthUser,
    State(state): State<ClientState>,
    Json(payload): Json<CreateClient>,
) -> Result<(StatusCode, Json<crate::domain::models::Client>), ApiError> {
    let client = state.create_client_uc.execute(auth_user.user_id, payload).await?;
    Ok((StatusCode::CREATED, Json(client)))
}

async fn get_client(
    auth_user: AuthUser,
    State(state): State<ClientState>,
    Path(client_id): Path<Uuid>,
) -> Result<Json<crate::domain::models::Client>, ApiError> {
    let client = state.get_client_uc.execute(auth_user.user_id, client_id).await?;
    Ok(Json(client))
}

async fn update_client(
    auth_user: AuthUser,
    State(state): State<ClientState>,
    Path(client_id): Path<Uuid>,
    Json(payload): Json<UpdateClient>,
) -> Result<Json<crate::domain::models::Client>, ApiError> {
    let client = state.update_client_uc.execute(auth_user.user_id, client_id, payload).await?;
    Ok(Json(client))
}

async fn delete_client(
    auth_user: AuthUser,
    State(state): State<ClientState>,
    Path(client_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.delete_client_uc.execute(auth_user.user_id, client_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_client_invoices(
    auth_user: AuthUser,
    State(state): State<ClientState>,
    Path(client_id): Path<Uuid>,
) -> Result<Json<Vec<crate::domain::models::InvoiceResponse>>, ApiError> {
    let invoices = state.get_client_invoices_uc.execute(auth_user.user_id, client_id).await?;
    Ok(Json(invoices))
}

async fn get_client_stats(
    auth_user: AuthUser,
    State(state): State<ClientState>,
) -> Result<Json<crate::domain::models::ClientStats>, ApiError> {
    let stats = state.get_client_stats_uc.execute(auth_user.user_id).await?;
    Ok(Json(stats))
}
