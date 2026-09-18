use sqlx::postgres::{PgPool, PgPoolOptions};

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("Database connection failed: {0}")]
    ConnectionError(#[from] sqlx::Error),
    #[error("Health check failed: database is unresponsive")]
    HealthCheckFailed,
    #[error("Database migration failed: {0}")]
    MigrationError(String),
}

pub async fn db_connection(database_url: &str) -> Result<PgPool, DbError> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    sqlx::query("SELECT 1")
        .execute(&pool)
        .await
        .map_err(|_| DbError::HealthCheckFailed)?;

    Ok(pool)
}

pub async fn run_migrations(pool: &PgPool) -> Result<(), DbError> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .map_err(|err| DbError::MigrationError(err.to_string()))?;
    Ok(())
}

pub async fn health_check(pool: &PgPool) -> Result<(), DbError> {
    sqlx::query("SELECT 1")
        .execute(pool)
        .await
        .map_err(|_| DbError::HealthCheckFailed)?;

    Ok(())
}
