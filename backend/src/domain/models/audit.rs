use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Type;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(type_name = "varchar")]
pub enum AuditAction {
    Create,
    Update,
    Delete,
    Login,
    Logout,
    Payment,
    Send,
    Refund,
}

impl std::fmt::Display for AuditAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditAction::Create => write!(f, "create"),
            AuditAction::Update => write!(f, "update"),
            AuditAction::Delete => write!(f, "delete"),
            AuditAction::Login => write!(f, "login"),
            AuditAction::Logout => write!(f, "logout"),
            AuditAction::Payment => write!(f, "payment"),
            AuditAction::Send => write!(f, "send"),
            AuditAction::Refund => write!(f, "refund"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(type_name = "varchar")]
pub enum AuditEntityType {
    User,
    Invoice,
    Client,
    Payment,
    Expense,
}

impl std::fmt::Display for AuditEntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditEntityType::User => write!(f, "user"),
            AuditEntityType::Invoice => write!(f, "invoice"),
            AuditEntityType::Client => write!(f, "client"),
            AuditEntityType::Payment => write!(f, "payment"),
            AuditEntityType::Expense => write!(f, "expense"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub action: AuditAction,
    pub entity_type: AuditEntityType,
    pub entity_id: Option<Uuid>,
    pub changes: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAuditLog {
    pub user_id: Option<Uuid>,
    pub action: AuditAction,
    pub entity_type: AuditEntityType,
    pub entity_id: Option<Uuid>,
    pub changes: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditListFilter {
    pub action: Option<AuditAction>,
    pub entity_type: Option<AuditEntityType>,
    pub user_id: Option<Uuid>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
