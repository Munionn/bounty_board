use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use uuid::Uuid;

use crate::app::AppState;
use crate::auth::AuthUser;
use crate::dto::{CreateSubmissionRequest, SubmissionResponse};
use crate::services::{bounty as bounty_service, submission as submission_service};

/// GET /api/bounties/{id}/submissions
pub async fn list_submissions(
    State(state): State<AppState>,
    Path(bounty_id): Path<Uuid>,
) -> Result<Json<Vec<SubmissionResponse>>, StatusCode> {
    let rows = submission_service::list_by_bounty(&state.db, bounty_id)
        .await
        .map_err(StatusCode::from)?;

    Ok(Json(
        rows.into_iter().map(SubmissionResponse::from).collect(),
    ))
}

/// POST /api/bounties/{id}/submissions
pub async fn create_submission(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(bounty_id): Path<Uuid>,
    Json(payload): Json<CreateSubmissionRequest>,
) -> Result<Json<SubmissionResponse>, StatusCode> {
    auth.ensure_wallet(&payload.submitter_wallet)?;

    let bounty = bounty_service::get_by_id(&state.db, bounty_id)
        .await
        .map_err(StatusCode::from)?;
    auth.ensure_bounty_claimer(&bounty)?;

    let row = submission_service::create(&state.db, bounty_id, &bounty, &payload)
        .await
        .map_err(StatusCode::from)?;

    Ok(Json(SubmissionResponse::from(row)))
}

/// GET /api/submissions/{id}
pub async fn get_submission(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<SubmissionResponse>, StatusCode> {
    let row = submission_service::get_by_id(&state.db, id)
        .await
        .map_err(StatusCode::from)?;

    Ok(Json(SubmissionResponse::from(row)))
}
