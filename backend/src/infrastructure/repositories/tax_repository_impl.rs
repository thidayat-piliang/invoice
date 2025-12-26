use async_trait::async_trait;
use uuid::Uuid;
use sqlx::{PgPool, Row};
use crate::domain::models::TaxSetting;
use crate::domain::repositories::TaxRepository;
use chrono::{DateTime, Utc};

pub struct TaxRepositoryImpl {
    pool: PgPool,
}

impl TaxRepositoryImpl {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // Helper function to map a row to TaxSetting
    fn map_row_to_tax_setting(row: &sqlx::postgres::PgRow) -> Result<TaxSetting, sqlx::Error> {
        Ok(TaxSetting {
            id: row.try_get("id")?,
            organization_id: row.try_get("organization_id")?,
            label: row.try_get("label")?,
            rate: row.try_get("rate")?,
            is_default: row.try_get::<bool, _>("is_default")?,
            is_active: row.try_get::<bool, _>("is_active")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

#[async_trait]
impl TaxRepository for TaxRepositoryImpl {
    async fn create(&self, tax_setting: TaxSetting) -> Result<TaxSetting, Box<dyn std::error::Error + Send + Sync>> {
        let result = sqlx::query(
            r#"
            INSERT INTO tax_settings (id, organization_id, label, rate, is_default, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
        )
        .bind(tax_setting.id)
        .bind(tax_setting.organization_id)
        .bind(&tax_setting.label)
        .bind(tax_setting.rate)
        .bind(tax_setting.is_default)
        .bind(tax_setting.is_active)
        .bind(tax_setting.created_at)
        .bind(tax_setting.updated_at)
        .fetch_one(&self.pool)
        .await
        .map(|row| Self::map_row_to_tax_setting(&row).unwrap())?;

        Ok(result)
    }

    async fn find_by_organization(&self, organization_id: Uuid) -> Result<Vec<TaxSetting>, Box<dyn std::error::Error + Send + Sync>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM tax_settings
            WHERE organization_id = $1
            ORDER BY is_default DESC, created_at DESC
            "#,
        )
        .bind(organization_id)
        .fetch_all(&self.pool)
        .await?;

        let result = rows
            .iter()
            .map(|row| Self::map_row_to_tax_setting(row).unwrap())
            .collect();

        Ok(result)
    }

    async fn find_by_id(&self, organization_id: Uuid, id: Uuid) -> Option<TaxSetting> {
        let result = sqlx::query(
            r#"
            SELECT * FROM tax_settings
            WHERE organization_id = $1 AND id = $2
            "#,
        )
        .bind(organization_id)
        .bind(id)
        .fetch_optional(&self.pool)
        .await;

        match result {
            Ok(Some(row)) => Self::map_row_to_tax_setting(&row).ok(),
            _ => None,
        }
    }

    async fn find_default(&self, organization_id: Uuid) -> Result<Option<TaxSetting>, Box<dyn std::error::Error + Send + Sync>> {
        let result = sqlx::query(
            r#"
            SELECT * FROM tax_settings
            WHERE organization_id = $1 AND is_default = true AND is_active = true
            "#,
        )
        .bind(organization_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.map(|row| Self::map_row_to_tax_setting(&row).unwrap()))
    }

    async fn update(&self, tax_setting: TaxSetting) -> Result<TaxSetting, Box<dyn std::error::Error + Send + Sync>> {
        let result = sqlx::query(
            r#"
            UPDATE tax_settings
            SET label = $1, rate = $2, is_default = $3, is_active = $4, updated_at = $5
            WHERE id = $6 AND organization_id = $7
            RETURNING *
            "#,
        )
        .bind(&tax_setting.label)
        .bind(tax_setting.rate)
        .bind(tax_setting.is_default)
        .bind(tax_setting.is_active)
        .bind(tax_setting.updated_at)
        .bind(tax_setting.id)
        .bind(tax_setting.organization_id)
        .fetch_one(&self.pool)
        .await
        .map(|row| Self::map_row_to_tax_setting(&row).unwrap())?;

        Ok(result)
    }

    async fn delete(&self, organization_id: Uuid, id: Uuid) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        sqlx::query(
            r#"
            DELETE FROM tax_settings
            WHERE organization_id = $1 AND id = $2
            "#,
        )
        .bind(organization_id)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn unset_all_defaults(&self, organization_id: Uuid) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        sqlx::query(
            r#"
            UPDATE tax_settings
            SET is_default = false
            WHERE organization_id = $1
            "#,
        )
        .bind(organization_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
