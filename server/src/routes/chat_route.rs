use axum::{
    Router,
    routing::{get, post},
};

use crate::app::AppState;
use crate::handlers::chat_file_handler::{
    complete_upload, get_download_url, get_file_status, register_file,
};
use crate::handlers::chat_handler::websocket_handler;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/ws", get(websocket_handler))
        .route("/files", post(register_file))
        .route("/files/{file_id}/complete", post(complete_upload))
        .route("/files/{file_id}/status", get(get_file_status))
        .route("/files/{file_id}/download", get(get_download_url))
}
