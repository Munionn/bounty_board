use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "bounty_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum BountyStatus {
    Open,
    Claimed,
    Closed,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Bounty {
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
