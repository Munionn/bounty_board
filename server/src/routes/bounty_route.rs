use axum::{
    Router,
    routing::{get, patch, post},
};

use crate::AppState;
use crate::handlers::bounty_handler::{
    confirm_bounty, create_bounty, delete_bounty, get_bounty, list_bounties, sync_bounty,
    update_bounty, update_bounty_status,
};
use crate::handlers::submission_handler::{create_submission, list_submissions};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_bounties).post(create_bounty))
        .route("/confirm", post(confirm_bounty))
        .route(
            "/{id}",
            get(get_bounty).put(update_bounty).delete(delete_bounty),
        )
        .route("/{id}/status", patch(update_bounty_status))
        .route("/{id}/sync", post(sync_bounty))
        .route(
            "/{id}/submissions",
            get(list_submissions).post(create_submission),
        )
}
