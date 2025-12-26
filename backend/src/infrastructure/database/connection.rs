use sqlx::{postgres::PgPoolOptions, PgPool, Error, migrate::MigrateError};
use std::time::Duration;

pub async fn create_pool(database_url: &str) -> Result<PgPool, Error> {
    PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(30))
        .connect(database_url)
        .await
}

pub async fn run_migrations(pool: &PgPool) -> Result<(), MigrateError> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
}
