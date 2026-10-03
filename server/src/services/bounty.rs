use sqlx::PgPool;
use uuid::Uuid;

use crate::dto::{CreateBountyRequest, ListBountiesQuery, UpdateBountyRequest};
use crate::error::ServiceError;
use crate::models::{Bounty, BountyStatus};

pub async fn list(pool: &PgPool, query: &ListBountiesQuery) -> Result<Vec<Bounty>, ServiceError> {
    let limit = query.limit.unwrap_or(50).clamp(1, 100);
    let offset = query.offset.unwrap_or(0).max(0);

    let bounties = sqlx::query_as::<_, Bounty>(
        r#"
        SELECT * FROM bounties
        WHERE ($1::text IS NULL OR status::text = $1)
          AND ($2::text IS NULL OR poster_wallet = $2)
          AND ($3::text IS NULL OR claimer_wallet = $3)
        ORDER BY created_at DESC
        LIMIT $4 OFFSET $5
        "#,
    )
    .bind(query.status.as_deref())
    .bind(query.poster.as_deref())
    .bind(query.claimer.as_deref())
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(bounties)
}

pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<Bounty, ServiceError> {
    sqlx::query_as::<_, Bounty>("SELECT * FROM bounties WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(ServiceError::NotFound)
}

pub async fn create(pool: &PgPool, payload: &CreateBountyRequest) -> Result<Bounty, ServiceError> {
    sqlx::query_as::<_, Bounty>(
        r#"
        INSERT INTO bounties (
            on_chain_id, poster_wallet, pda_address, title, description,
            amount_lamports, status, tx_signature
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING *
        "#,
    )
    .bind(payload.on_chain_id)
    .bind(&payload.poster_wallet)
    .bind(&payload.pda_address)
    .bind(&payload.title)
    .bind(&payload.description)
    .bind(payload.amount_lamports)
    .bind(BountyStatus::Open)
    .bind(&payload.tx_signature)
    .fetch_one(pool)
    .await
    .map_err(ServiceError::from_db)
}

/// Atomically assign a claimer. Fails if the bounty is no longer open or already claimed.
pub async fn claim(pool: &PgPool, id: Uuid, claimer_wallet: &str) -> Result<Bounty, ServiceError> {
    sqlx::query_as::<_, Bounty>(
        r#"
        UPDATE bounties
        SET
            claimer_wallet = $2,
            status = 'claimed',
            updated_at = NOW()
        WHERE id = $1
          AND status = 'open'
          AND claimer_wallet IS NULL
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(claimer_wallet)
    .fetch_optional(pool)
    .await?
    .ok_or(ServiceError::Conflict)
}

pub async fn update(
    pool: &PgPool,
    id: Uuid,
    payload: &UpdateBountyRequest,
) -> Result<Bounty, ServiceError> {
    sqlx::query_as::<_, Bounty>(
        r#"
        UPDATE bounties
        SET
            title = COALESCE($2, title),
            description = COALESCE($3, description),
            claimer_wallet = COALESCE($4, claimer_wallet),
            amount_lamports = COALESCE($5, amount_lamports),
            status = COALESCE($6, status),
            submission_uri = COALESCE($7, submission_uri),
            tx_signature = COALESCE($8, tx_signature)
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(&payload.title)
    .bind(&payload.description)
    .bind(&payload.claimer_wallet)
    .bind(payload.amount_lamports)
    .bind(payload.status)
    .bind(&payload.submission_uri)
    .bind(&payload.tx_signature)
    .fetch_optional(pool)
    .await?
    .ok_or(ServiceError::NotFound)
}

pub async fn update_status(
    pool: &PgPool,
    id: Uuid,
    status: BountyStatus,
) -> Result<Bounty, ServiceError> {
    sqlx::query_as::<_, Bounty>(
        r#"
        UPDATE bounties
        SET status = $2
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(status)
    .fetch_optional(pool)
    .await?
    .ok_or(ServiceError::NotFound)
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), ServiceError> {
    let result = sqlx::query("DELETE FROM bounties WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(ServiceError::NotFound);
    }

    Ok(())
}
