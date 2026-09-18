mod auth_route;
mod bounty_route;
mod submission_route;
mod user_route;

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::get,
};
use serde::Serialize;
use sqlx::PgPool;

use crate::db;
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .nest("/api/auth", auth_route::router())
        .nest("/api/users", user_route::router())
        .nest("/api/bounties", bounty_route::router())
        .nest("/api/submissions", submission_route::router())
}

async fn root() -> &'static str {
    "server is ok"
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    database: &'static str,
}

async fn health(State(state): State<AppState>) -> Result<Json<HealthResponse>, StatusCode> {
    check_database(&state.db).await?;
    Ok(Json(HealthResponse {
        status: "ok",
        database: "up",
    }))
}

async fn check_database(pool: &PgPool) -> Result<(), StatusCode> {
    db::health_check(pool).await.map_err(|err| {
        tracing::error!("database health check failed: {err}");
        StatusCode::SERVICE_UNAVAILABLE
    })
}
