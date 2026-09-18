use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use crate::models::BountyStatus;

/// Body for `POST /api/bounties`
#[derive(Debug, Clone, Deserialize)]
pub struct CreateBountyRequest {
    pub on_chain_id: i64,
    pub poster_wallet: String,
    pub pda_address: String,
    pub title: String,
    #[serde(default)]
    pub description: String,
    pub amount_lamports: i64,
    /// Optional until a tx is confirmed / indexed.
    pub tx_signature: Option<String>,
}

/// Body for `PATCH/PUT /api/bounties/{id}`
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateBountyRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub claimer_wallet: Option<String>,
    pub amount_lamports: Option<i64>,
    pub status: Option<BountyStatus>,
    pub submission_uri: Option<String>,
    pub tx_signature: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BountyResponse {
    pub id: Uuid,
    pub on_chain_id: i64,
    pub poster_wallet: String,
    pub claimer_wallet: Option<String>,
    pub pda_address: String,
    pub title: String,
    pub description: String,
    pub amount_lamports: i64,
    pub status: BountyStatus,
    pub submission_uri: Option<String>,
    pub tx_signature: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<crate::models::Bounty> for BountyResponse {
    fn from(bounty: crate::models::Bounty) -> Self {
        Self {
            id: bounty.id,
            on_chain_id: bounty.on_chain_id,
            poster_wallet: bounty.poster_wallet,
            claimer_wallet: bounty.claimer_wallet,
            pda_address: bounty.pda_address,
            title: bounty.title,
            description: bounty.description,
            amount_lamports: bounty.amount_lamports,
            status: bounty.status,
            submission_uri: bounty.submission_uri,
            tx_signature: bounty.tx_signature,
            created_at: bounty.created_at,
            updated_at: bounty.updated_at,
        }
    }
}
