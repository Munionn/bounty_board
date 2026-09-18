use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::User;

#[derive(Debug, Clone, Deserialize)]
pub struct CreateUserRequest {
    pub wallet_address: String,
    pub display_name: Option<String>,
    pub bio: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateUserRequest {
    pub display_name: Option<String>,
    pub bio: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub wallet_address: String,
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub reputation: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            wallet_address: user.wallet_address,
            display_name: user.display_name,
            bio: user.bio,
            reputation: user.reputation,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}
