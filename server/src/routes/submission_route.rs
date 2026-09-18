use axum::{Router, routing::get};

use crate::AppState;
use crate::handlers::submission_handler::get_submission;

pub fn router() -> Router<AppState> {
    Router::new().route("/{id}", get(get_submission))
}
