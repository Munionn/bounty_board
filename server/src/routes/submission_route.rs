use axum::{routing::get, Router};

use crate::handlers::submission_handler::get_submission;
use crate::app::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/{id}", get(get_submission))
}
