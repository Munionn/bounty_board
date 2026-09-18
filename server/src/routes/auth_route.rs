use axum::{
    Router,
    routing::{get, post},
};

use crate::AppState;
use crate::handlers::auth_handler::{get_nonce, me, verify_signature};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/nonce", get(get_nonce))
        .route("/verify", post(verify_signature))
        .route("/me", get(me))
}
