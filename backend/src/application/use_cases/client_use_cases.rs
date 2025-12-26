use std::sync::Arc;
use uuid::Uuid;
use thiserror::Error;

use crate::domain::services::ClientService;
use crate::domain::models::{Client, ClientResponse, ClientStats, CreateClient, UpdateClient};

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("Client not found")]
    NotFound,
    #[error("Database error: {0}")]
    DatabaseError(String),
}

impl From<sqlx::Error> for ClientError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => ClientError::NotFound,
            _ => ClientError::DatabaseError(err.to_string()),
        }
    }
}

// CreateClientUseCase
#[derive(Clone)]
pub struct CreateClientUseCase {
    client_service: Arc<ClientService>,
}

impl CreateClientUseCase {
    pub fn new(client_service: Arc<ClientService>) -> Self {
        Self { client_service }
    }

    pub async fn execute(&self, user_id: Uuid, create: CreateClient) -> Result<Client, ClientError> {
        Ok(self.client_service.create_client(user_id, create).await?)
    }
}

// GetClientUseCase
#[derive(Clone)]
pub struct GetClientUseCase {
    client_service: Arc<ClientService>,
}

impl GetClientUseCase {
    pub fn new(client_service: Arc<ClientService>) -> Self {
        Self { client_service }
    }

    pub async fn execute(&self, user_id: Uuid, client_id: Uuid) -> Result<Client, ClientError> {
        let client = self.client_service.get_client(user_id, client_id).await?;
        client.ok_or(ClientError::NotFound)
    }
}

// ListClientsUseCase
#[derive(Clone)]
pub struct ListClientsUseCase {
    client_service: Arc<ClientService>,
}

impl ListClientsUseCase {
    pub fn new(client_service: Arc<ClientService>) -> Self {
        Self { client_service }
    }

    pub async fn execute(
        &self,
        user_id: Uuid,
        search: Option<String>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<ClientResponse>, ClientError> {
        Ok(self.client_service.list_clients(user_id, search, limit, offset).await?)
    }
}

// UpdateClientUseCase
#[derive(Clone)]
pub struct UpdateClientUseCase {
    client_service: Arc<ClientService>,
}

impl UpdateClientUseCase {
    pub fn new(client_service: Arc<ClientService>) -> Self {
        Self { client_service }
    }

    pub async fn execute(
        &self,
        user_id: Uuid,
        client_id: Uuid,
        update: UpdateClient,
    ) -> Result<Client, ClientError> {
        Ok(self.client_service.update_client(user_id, client_id, update).await?)
    }
}

// DeleteClientUseCase
#[derive(Clone)]
pub struct DeleteClientUseCase {
    client_service: Arc<ClientService>,
}

impl DeleteClientUseCase {
    pub fn new(client_service: Arc<ClientService>) -> Self {
        Self { client_service }
    }

    pub async fn execute(&self, user_id: Uuid, client_id: Uuid) -> Result<(), ClientError> {
        Ok(self.client_service.delete_client(user_id, client_id).await?)
    }
}

// GetClientInvoicesUseCase
#[derive(Clone)]
pub struct GetClientInvoicesUseCase {
    client_service: Arc<ClientService>,
}

impl GetClientInvoicesUseCase {
    pub fn new(client_service: Arc<ClientService>) -> Self {
        Self { client_service }
    }

    pub async fn execute(
        &self,
        user_id: Uuid,
        client_id: Uuid,
    ) -> Result<Vec<crate::domain::models::InvoiceResponse>, ClientError> {
        Ok(self.client_service.get_client_invoices(user_id, client_id).await?)
    }
}

// GetClientStatsUseCase
#[derive(Clone)]
pub struct GetClientStatsUseCase {
    client_service: Arc<ClientService>,
}

impl GetClientStatsUseCase {
    pub fn new(client_service: Arc<ClientService>) -> Self {
        Self { client_service }
    }

    pub async fn execute(&self, user_id: Uuid) -> Result<ClientStats, ClientError> {
        Ok(self.client_service.get_stats(user_id).await?)
    }
}
