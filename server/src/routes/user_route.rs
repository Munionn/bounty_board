use axum::{
    Router,
    routing::get,
};

use crate::AppState;
use crate::handlers::users_handler::{
    create_user, delete_user, get_user, get_user_by_wallet, list_users, update_user,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_users).post(create_user))
        // static path before /{id}
        .route("/wallet/{wallet}", get(get_user_by_wallet))
        .route("/{id}", get(get_user).put(update_user).delete(delete_user))
}
