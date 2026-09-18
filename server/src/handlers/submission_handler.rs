use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use uuid::Uuid;

use crate::AppState;
use crate::dto::{CreateSubmissionRequest, SubmissionResponse};
use crate::models::Submission;

/// GET /api/bounties/{id}/submissions
pub async fn list_submissions(
    State(state): State<AppState>,
    Path(bounty_id): Path<Uuid>,
) -> Result<Json<Vec<SubmissionResponse>>, StatusCode> {
    let rows = sqlx::query_as::<_, Submission>(
        "SELECT * FROM submissions WHERE bounty_id = $1 ORDER BY created_at DESC",
    )
    .bind(bounty_id)
    .fetch_all(&state.db)
    .await
    .map_err(|err| {
        tracing::error!("list_submissions failed: {err}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(
        rows.into_iter().map(SubmissionResponse::from).collect(),
    ))
}

/// POST /api/bounties/{id}/submissions
pub async fn create_submission(
    State(state): State<AppState>,
    Path(bounty_id): Path<Uuid>,
    Json(payload): Json<CreateSubmissionRequest>,
) -> Result<Json<SubmissionResponse>, StatusCode> {
    let row = sqlx::query_as::<_, Submission>(
        r#"
        INSERT INTO submissions (bounty_id, submitter_wallet, submission_uri, note)
        VALUES ($1, $2, $3, $4)
        RETURNING *
        "#,
    )
    .bind(bounty_id)
    .bind(&payload.submitter_wallet)
    .bind(&payload.submission_uri)
    .bind(&payload.note)
    .fetch_one(&state.db)
    .await
    .map_err(|err| {
        tracing::error!("create_submission failed: {err}");
        if let sqlx::Error::Database(db_err) = &err {
            if db_err.constraint().is_some() {
                return StatusCode::CONFLICT;
            }
        }
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(SubmissionResponse::from(row)))
}

/// GET /api/submissions/{id}
pub async fn get_submission(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<SubmissionResponse>, StatusCode> {
    let row = sqlx::query_as::<_, Submission>("SELECT * FROM submissions WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .map_err(|err| {
            tracing::error!("get_submission failed: {err}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(SubmissionResponse::from(row)))
}
