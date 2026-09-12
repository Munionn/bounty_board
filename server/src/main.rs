use axum::{extract::State, http::StatusCode, routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tower_http::cors::CorsLayer;
use tracing_subscriber::EnvFilter;

#[derive(Clone, Default)]
struct AppState {
    /// Off-chain bounty metadata keyed by on-chain bounty id.
    bounties: Arc<Mutex<HashMap<u64, BountyMeta>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BountyMeta {
    id: u64,
    title: String,
    description: String,
    poster: String,
    amount_lamports: u64,
}

#[derive(Debug, Deserialize)]
struct CreateBountyRequest {
    id: u64,
    title: String,
    description: String,
    poster: String,
    amount_lamports: u64,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse().unwrap()))
        .init();

    let state = AppState::default();

    let app = Router::new()
        .route("/health", get(health))
        .route("/", get(|| async { "server is ok" }))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("failed to bind :3000");

    tracing::info!("server listening on http://0.0.0.0:3000");
    axum::serve(listener, app).await.expect("server failed");
}

async fn health() -> &'static str {
    "ok"
}
