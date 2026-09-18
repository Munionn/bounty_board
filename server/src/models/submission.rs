use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Submission {
    pub id: Uuid,
    pub bounty_id: Uuid,
    pub submitter_wallet: String,
    pub submission_uri: String,
    pub note: Option<String>,
    pub created_at: DateTime<Utc>,
}
