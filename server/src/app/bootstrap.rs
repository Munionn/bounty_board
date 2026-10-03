use std::sync::Arc;

use axum::Router;
use tower_http::cors::CorsLayer;
use tracing_subscriber::EnvFilter;

use crate::app::chat_hub::ChatHub;
use crate::app::state::AppState;
use crate::auth::NonceStore;
use crate::db;
use crate::storage::StorageState;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub jwt_secret: String,
    pub host: String,
    pub port: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        let database_url =
            std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
        let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| {
            tracing::warn!("JWT_SECRET not set; using insecure default for local dev");
            "dev-only-change-me".into()
        });
        let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into());
        let port = std::env::var("PORT").unwrap_or_else(|_| "3000".into());

        Self {
            database_url,
            jwt_secret,
            host,
            port,
        }
    }
}

pub async fn init_state(config: &AppConfig) -> Result<AppState, db::DbError> {
    let db = db::db_connection(&config.database_url).await?;
    db::run_migrations(&db).await?;
    tracing::info!("connected to postgres and migrations applied");

    let storage = StorageState::from_env();
    tracing::info!(
        bucket = %storage.default_bucket,
        "connected to minio"
    );

    Ok(AppState::new(
        db,
        config.jwt_secret.clone(),
        storage,
        Arc::new(NonceStore::default()),
        ChatHub::default(),
    ))
}

pub fn build_router(state: AppState) -> Router {
    crate::routes::router()
        .layer(CorsLayer::permissive())
        .with_state(state)
}

pub fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse().unwrap()))
        .init();
}

pub async fn run(config: AppConfig) -> Result<(), db::DbError> {
    init_tracing();

    let state = init_state(&config).await?;
    let app = build_router(state);

    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|err| panic!("failed to bind {addr}: {err}"));

    tracing::info!("server listening on http://{addr}");
    axum::serve(listener, app).await.expect("server failed");

    Ok(())
}
