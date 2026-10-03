use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};

use crate::app::AppState;
use crate::auth::AuthUser;
use crate::dto::{
    AuthNonceQuery, AuthNonceResponse, AuthVerifyRequest, AuthVerifyResponse, UserResponse,
};
use crate::services::{auth as auth_service, user as user_service};

/// GET /api/auth/nonce?wallet=...
pub async fn get_nonce(
    State(state): State<AppState>,
    Query(query): Query<AuthNonceQuery>,
) -> Result<Json<AuthNonceResponse>, StatusCode> {
    let response = auth_service::create_nonce(&state, &query)
        .await
        .map_err(StatusCode::from)?;

    Ok(Json(response))
}

/// POST /api/auth/verify
pub async fn verify_signature(
    State(state): State<AppState>,
    Json(payload): Json<AuthVerifyRequest>,
) -> Result<Json<AuthVerifyResponse>, StatusCode> {
    let response = auth_service::verify_and_issue_token(&state, &payload)
        .await
        .map_err(StatusCode::from)?;

    Ok(Json(response))
}

/// GET /api/auth/me
pub async fn me(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<UserResponse>, StatusCode> {
    let user = user_service::get_by_wallet(&state.db, auth.wallet())
        .await
        .map_err(StatusCode::from)?;

    Ok(Json(UserResponse::from(user)))
}
