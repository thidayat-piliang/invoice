use std::sync::Arc;
use uuid::Uuid;

use crate::infrastructure::repositories::ClientRepository;
use crate::domain::models::{Client, ClientResponse, ClientStats, CreateClient, UpdateClient};

#[derive(Clone)]
pub struct ClientService {
    client_repo: Arc<ClientRepository>,
}

impl ClientService {
    pub fn new(client_repo: Arc<ClientRepository>) -> Self {
        Self { client_repo }
    }

    pub async fn create_client(&self, user_id: Uuid, create: CreateClient) -> Result<Client, sqlx::Error> {
        let billing_address = create.billing_address
            .map(|addr| serde_json::to_value(addr).unwrap_or(serde_json::Value::Null));

        self.client_repo.create(
            user_id,
            create.name,
            create.email,
            create.phone,
            create.company_name,
            billing_address,
            create.payment_terms,
            create.tax_exempt,
            create.notes,
        ).await
    }

    pub async fn get_client(&self, user_id: Uuid, client_id: Uuid) -> Result<Option<Client>, sqlx::Error> {
        self.client_repo.find_by_id(user_id, client_id).await
    }

    pub async fn list_clients(
        &self,
        user_id: Uuid,
        search: Option<String>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<ClientResponse>, sqlx::Error> {
        self.client_repo.list(user_id, search, limit, offset).await
    }

    pub async fn update_client(
        &self,
        user_id: Uuid,
        client_id: Uuid,
        update: UpdateClient,
    ) -> Result<Client, sqlx::Error> {
        let billing_address = update.billing_address
            .map(|addr| serde_json::to_value(addr).unwrap_or(serde_json::Value::Null));

        self.client_repo.update(
            user_id,
            client_id,
            update.name,
            update.email,
            update.phone,
            update.company_name,
            billing_address,
            update.payment_terms,
            update.tax_exempt,
            update.notes,
        ).await
    }

    pub async fn delete_client(&self, user_id: Uuid, client_id: Uuid) -> Result<(), sqlx::Error> {
        self.client_repo.delete(user_id, client_id).await
    }

    pub async fn get_stats(&self, user_id: Uuid) -> Result<ClientStats, sqlx::Error> {
        self.client_repo.get_stats(user_id).await
    }

    pub async fn get_client_invoices(
        &self,
        user_id: Uuid,
        client_id: Uuid,
    ) -> Result<Vec<crate::domain::models::InvoiceResponse>, sqlx::Error> {
        self.client_repo.get_client_invoices(user_id, client_id).await
    }
}
