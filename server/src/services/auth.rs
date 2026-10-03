use std::time::Instant;

use uuid::Uuid;

use crate::app::AppState;
use crate::auth::{NonceRecord, issue_token, verify_wallet_signature};
use crate::dto::{AuthNonceQuery, AuthNonceResponse, AuthVerifyRequest, AuthVerifyResponse};
use crate::error::ServiceError;
use crate::services::user;

pub async fn create_nonce(state: &AppState, query: &AuthNonceQuery) -> Result<AuthNonceResponse, ServiceError> {
    if query.wallet.trim().is_empty() {
        return Err(ServiceError::BadRequest("wallet is required".into()));
    }

    let nonce = Uuid::new_v4().to_string();
    let message = format!(
        "Sign in to Bounty Board\nWallet: {}\nNonce: {}",
        query.wallet, nonce
    );

    state
        .nonces
        .put(
            query.wallet.clone(),
            NonceRecord {
                nonce: nonce.clone(),
                message: message.clone(),
                created_at: Instant::now(),
            },
        )
        .await;

    Ok(AuthNonceResponse {
        wallet: query.wallet.clone(),
        message,
        nonce,
    })
}

pub async fn verify_and_issue_token(
    state: &AppState,
    payload: &AuthVerifyRequest,
) -> Result<AuthVerifyResponse, ServiceError> {
    let record = state
        .nonces
        .take(&payload.wallet)
        .await
        .ok_or(ServiceError::Unauthorized)?;

    if record.message != payload.message || !payload.message.contains(&record.nonce) {
        return Err(ServiceError::Unauthorized);
    }

    verify_wallet_signature(&payload.wallet, &payload.message, &payload.signature).map_err(
        |err| {
            tracing::warn!("wallet signature verify failed: {err:?}");
            ServiceError::Unauthorized
        },
    )?;

    let user = user::upsert_by_wallet(&state.db, &payload.wallet).await?;

    let token = issue_token(
        &payload.wallet,
        &user.id.to_string(),
        &state.jwt_secret,
    )
    .map_err(|err| {
        tracing::error!("jwt issue failed: {err}");
        ServiceError::Internal
    })?;

    Ok(AuthVerifyResponse {
        token,
        wallet: payload.wallet.clone(),
    })
}
