use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts, HeaderMap, StatusCode},
};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::app::AppState;
use crate::models::{Bounty, BountyStatus};

const NONCE_TTL: Duration = Duration::from_secs(5 * 60);
const JWT_TTL_SECS: i64 = 60 * 60 * 24 * 7; // 7 days

#[derive(Clone)]
pub struct NonceRecord {
    pub nonce: String,
    pub message: String,
    pub created_at: Instant,
}

#[derive(Default)]
pub struct NonceStore {
    inner: Mutex<HashMap<String, NonceRecord>>,
}

impl NonceStore {
    pub async fn put(&self, wallet: String, record: NonceRecord) {
        let mut map = self.inner.lock().await;
        map.retain(|_, r| r.created_at.elapsed() < NONCE_TTL);
        map.insert(wallet, record);
    }

    pub async fn take(&self, wallet: &str) -> Option<NonceRecord> {
        let mut map = self.inner.lock().await;
        map.retain(|_, r| r.created_at.elapsed() < NONCE_TTL);
        map.remove(wallet)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub wallet: String,
    pub exp: i64,
}

pub fn issue_token(wallet: &str, user_id: &str, secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let exp = chrono::Utc::now().timestamp() + JWT_TTL_SECS;
    let claims = Claims {
        sub: user_id.to_string(),
        wallet: wallet.to_string(),
        exp,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

pub fn verify_token(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;
    Ok(data.claims)
}

pub fn verify_wallet_signature(
    wallet: &str,
    message: &str,
    signature_b58: &str,
) -> Result<(), StatusAuthError> {
    let pubkey_bytes = bs58::decode(wallet)
        .into_vec()
        .map_err(|_| StatusAuthError::InvalidWallet)?;
    if pubkey_bytes.len() != 32 {
        return Err(StatusAuthError::InvalidWallet);
    }
    let mut key_bytes = [0u8; 32];
    key_bytes.copy_from_slice(&pubkey_bytes);

    let sig_bytes = bs58::decode(signature_b58)
        .into_vec()
        .map_err(|_| StatusAuthError::InvalidSignature)?;
    if sig_bytes.len() != 64 {
        return Err(StatusAuthError::InvalidSignature);
    }
    let mut sig_arr = [0u8; 64];
    sig_arr.copy_from_slice(&sig_bytes);

    let verifying_key =
        VerifyingKey::from_bytes(&key_bytes).map_err(|_| StatusAuthError::InvalidWallet)?;
    let signature = Signature::from_bytes(&sig_arr);

    verifying_key
        .verify(message.as_bytes(), &signature)
        .map_err(|_| StatusAuthError::InvalidSignature)
}

#[derive(Debug)]
pub enum StatusAuthError {
    InvalidWallet,
    InvalidSignature,
}

pub type SharedNonceStore = Arc<NonceStore>;

pub fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(header::AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
}

/// Authenticated user extracted from `Authorization: Bearer <jwt>`.
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub claims: Claims,
}

impl AuthUser {
    pub fn wallet(&self) -> &str {
        &self.claims.wallet
    }

    pub fn user_id(&self) -> &str {
        &self.claims.sub
    }

    pub fn ensure_wallet(&self, wallet: &str) -> Result<(), StatusCode> {
        if self.wallet() != wallet {
            return Err(StatusCode::FORBIDDEN);
        }
        Ok(())
    }

    pub fn ensure_bounty_poster(&self, bounty: &Bounty) -> Result<(), StatusCode> {
        if self.wallet() != bounty.poster_wallet {
            return Err(StatusCode::FORBIDDEN);
        }
        Ok(())
    }

    pub fn ensure_bounty_participant(&self, bounty: &Bounty) -> Result<(), StatusCode> {
        if self.wallet() == bounty.poster_wallet {
            return Ok(());
        }
        if bounty.claimer_wallet.as_deref() == Some(self.wallet()) {
            return Ok(());
        }
        Err(StatusCode::FORBIDDEN)
    }

    /// Only the assigned worker may deliver work on a claimed bounty.
    pub fn ensure_bounty_claimer(&self, bounty: &Bounty) -> Result<(), StatusCode> {
        if bounty.status != BountyStatus::Claimed {
            return Err(StatusCode::CONFLICT);
        }
        if bounty.claimer_wallet.as_deref() != Some(self.wallet()) {
            return Err(StatusCode::FORBIDDEN);
        }
        Ok(())
    }
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = bearer_token(&parts.headers).ok_or(StatusCode::UNAUTHORIZED)?;
        let claims =
            verify_token(token, &state.jwt_secret).map_err(|_| StatusCode::UNAUTHORIZED)?;
        Ok(Self { claims })
    }
}

pub fn can_claim_bounty(auth: &AuthUser, bounty: &Bounty, claimer_wallet: &str) -> Result<(), StatusCode> {
    auth.ensure_wallet(claimer_wallet)?;
    if bounty.status != BountyStatus::Open {
        return Err(StatusCode::CONFLICT);
    }
    if bounty.claimer_wallet.is_some() {
        return Err(StatusCode::CONFLICT);
    }
    if auth.wallet() == bounty.poster_wallet {
        return Err(StatusCode::FORBIDDEN);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jwt_roundtrip() {
        let token = issue_token("wallet123", "user-uuid", "test-secret").expect("issue");
        let claims = verify_token(&token, "test-secret").expect("verify");
        assert_eq!(claims.wallet, "wallet123");
        assert_eq!(claims.sub, "user-uuid");
    }
}
