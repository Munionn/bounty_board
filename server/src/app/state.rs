use sqlx::PgPool;

use super::chat_hub::ChatHub;
use crate::auth::SharedNonceStore;
use crate::storage::StorageState;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub nonces: SharedNonceStore,
    pub jwt_secret: String,
    pub storage: StorageState,
    pub chat_hub: ChatHub,
}

impl AppState {
    pub fn new(
        db: PgPool,
        jwt_secret: impl Into<String>,
        storage: StorageState,
        nonces: SharedNonceStore,
        chat_hub: ChatHub,
    ) -> Self {
        Self {
            db,
            nonces,
            jwt_secret: jwt_secret.into(),
            storage,
            chat_hub,
        }
    }
}
