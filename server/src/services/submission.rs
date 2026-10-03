use sqlx::PgPool;
use uuid::Uuid;

use crate::dto::CreateSubmissionRequest;
use crate::error::ServiceError;
use crate::models::{Bounty, BountyStatus, Submission};

pub fn ensure_can_submit(bounty: &Bounty, submitter_wallet: &str) -> Result<(), ServiceError> {
    if bounty.status != BountyStatus::Claimed {
        return Err(ServiceError::BadRequest(
            "bounty must be claimed before submitting work".into(),
        ));
    }
    if bounty.claimer_wallet.as_deref() != Some(submitter_wallet) {
        return Err(ServiceError::Forbidden);
    }
    Ok(())
}

pub async fn list_by_bounty(pool: &PgPool, bounty_id: Uuid) -> Result<Vec<Submission>, ServiceError> {
    let rows = sqlx::query_as::<_, Submission>(
        "SELECT * FROM submissions WHERE bounty_id = $1 ORDER BY created_at DESC",
    )
    .bind(bounty_id)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<Submission, ServiceError> {
    sqlx::query_as::<_, Submission>("SELECT * FROM submissions WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(ServiceError::NotFound)
}

pub async fn create(
    pool: &PgPool,
    bounty_id: Uuid,
    bounty: &Bounty,
    payload: &CreateSubmissionRequest,
) -> Result<Submission, ServiceError> {
    ensure_can_submit(bounty, &payload.submitter_wallet)?;

    let uri = payload.submission_uri.as_deref().map(str::trim).filter(|s| !s.is_empty());
    let note = payload.note.as_deref().map(str::trim).filter(|s| !s.is_empty());

    if uri.is_none() && note.is_none() {
        return Err(ServiceError::BadRequest(
            "submission must include a link or a note".into(),
        ));
    }

    sqlx::query_as::<_, Submission>(
        r#"
        INSERT INTO submissions (bounty_id, submitter_wallet, submission_uri, note)
        VALUES ($1, $2, $3, $4)
        RETURNING *
        "#,
    )
    .bind(bounty_id)
    .bind(&payload.submitter_wallet)
    .bind(uri)
    .bind(note)
    .fetch_one(pool)
    .await
    .map_err(ServiceError::from_db)
}
