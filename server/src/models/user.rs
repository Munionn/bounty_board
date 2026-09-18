use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub wallet_address: String,
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub reputation: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
