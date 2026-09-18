use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};

use crate::AppState;
use crate::dto::{AuthNonceQuery, AuthNonceResponse, AuthVerifyRequest, AuthVerifyResponse, UserResponse};

/// GET /api/auth/nonce?wallet=...
pub async fn get_nonce(
    State(_state): State<AppState>,
    Query(query): Query<AuthNonceQuery>,
) -> Result<Json<AuthNonceResponse>, StatusCode> {
    // TODO: persist nonce in DB/Redis with TTL
    let nonce = uuid::Uuid::new_v4().to_string();
    let message = format!(
        "Sign in to Bounty Board\nWallet: {}\nNonce: {}",
        query.wallet, nonce
    );

    Ok(Json(AuthNonceResponse {
        wallet: query.wallet,
        message,
        nonce,
    }))
}

/// POST /api/auth/verify
pub async fn verify_signature(
    State(_state): State<AppState>,
    Json(_payload): Json<AuthVerifyRequest>,
) -> Result<Json<AuthVerifyResponse>, StatusCode> {
    // TODO: verify ed25519 signature, upsert user, issue JWT
    Err(StatusCode::NOT_IMPLEMENTED)
}

/// GET /api/me
pub async fn me(State(_state): State<AppState>) -> Result<Json<UserResponse>, StatusCode> {
    // TODO: read wallet from JWT middleware / extensions
    Err(StatusCode::NOT_IMPLEMENTED)
}
