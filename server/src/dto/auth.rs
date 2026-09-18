use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct AuthNonceQuery {
    pub wallet: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthNonceResponse {
    pub wallet: String,
    pub message: String,
    pub nonce: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthVerifyRequest {
    pub wallet: String,
    pub message: String,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthVerifyResponse {
    pub token: String,
    pub wallet: String,
}
