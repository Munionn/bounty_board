use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::app::AppState;
use crate::auth::AuthUser;
use crate::dto::{
    CompleteUploadRequest, FileDownloadResponse, FileStatusResponse, RegisterFileRequest,
    RegisterFileResponse,
};
use crate::services::{bounty as bounty_service, chat as chat_service, chat_attachment};

#[derive(Debug, Deserialize)]
pub struct DownloadQuery {
    #[serde(default)]
    pub preview: bool,
}

/// POST /api/bounties/{id}/chat/files
pub async fn register_file(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(bounty_id): Path<Uuid>,
    Json(payload): Json<RegisterFileRequest>,
) -> Result<Json<RegisterFileResponse>, StatusCode> {
    let bounty = bounty_service::get_by_id(&state.db, bounty_id)
        .await
        .map_err(StatusCode::from)?;

    let chat = chat_service::ensure_chat_for_bounty(&state.db, bounty_id)
        .await
        .map_err(StatusCode::from)?;

    let response = chat_attachment::register_attachment(
        &state.db,
        &state.storage,
        &bounty,
        chat.id,
        auth.wallet(),
        &payload,
    )
    .await
    .map_err(StatusCode::from)?;

    Ok(Json(response))
}

/// POST /api/bounties/{id}/chat/files/{file_id}/complete
pub async fn complete_upload(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((bounty_id, file_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<CompleteUploadRequest>,
) -> Result<(StatusCode, Json<FileStatusResponse>), StatusCode> {
    let bounty = bounty_service::get_by_id(&state.db, bounty_id)
        .await
        .map_err(StatusCode::from)?;

    let response = chat_attachment::complete_upload(
        state,
        &bounty,
        file_id,
        auth.wallet(),
        payload,
    )
    .await
    .map_err(StatusCode::from)?;

    Ok((StatusCode::ACCEPTED, Json(response)))
}

/// GET /api/bounties/{id}/chat/files/{file_id}/status
pub async fn get_file_status(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((bounty_id, file_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<FileStatusResponse>, StatusCode> {
    let bounty = bounty_service::get_by_id(&state.db, bounty_id)
        .await
        .map_err(StatusCode::from)?;

    let response = chat_attachment::get_file_status(&state.db, &bounty, file_id, auth.wallet())
        .await
        .map_err(StatusCode::from)?;

    Ok(Json(response))
}

/// GET /api/bounties/{id}/chat/files/{file_id}/download
pub async fn get_download_url(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((bounty_id, file_id)): Path<(Uuid, Uuid)>,
    Query(query): Query<DownloadQuery>,
) -> Result<Json<FileDownloadResponse>, StatusCode> {
    let bounty = bounty_service::get_by_id(&state.db, bounty_id)
        .await
        .map_err(StatusCode::from)?;

    let response = chat_attachment::get_download_url(
        &state.db,
        &state.storage,
        &bounty,
        file_id,
        auth.wallet(),
        query.preview,
    )
    .await
    .map_err(StatusCode::from)?;

    Ok(Json(response))
}
