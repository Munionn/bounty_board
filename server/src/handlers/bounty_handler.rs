use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::AppState;
use crate::dto::{BountyResponse, CreateBountyRequest, UpdateBountyRequest};
use crate::models::{Bounty, BountyStatus};

#[derive(Debug, Deserialize)]
pub struct ListBountiesQuery {
    pub status: Option<String>,
    pub poster: Option<String>,
    pub claimer: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct ConfirmBountyRequest {
    pub tx_signature: String,
    pub pda_address: Option<String>,
}

/// GET /api/bounties?status=&poster=&claimer=
pub async fn list_bounties(
    State(state): State<AppState>,
    Query(query): Query<ListBountiesQuery>,
) -> Result<Json<Vec<BountyResponse>>, StatusCode> {
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
    .fetch_all(&state.db)
    .await
    .map_err(|err| {
        tracing::error!("list_bounties failed: {err}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(
        bounties.into_iter().map(BountyResponse::from).collect(),
    ))
}

pub async fn create_bounty(
    State(state): State<AppState>,
    Json(payload): Json<CreateBountyRequest>,
) -> Result<Json<BountyResponse>, StatusCode> {
    let bounty = sqlx::query_as::<_, Bounty>(
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
    .fetch_one(&state.db)
    .await
    .map_err(|err| {
        tracing::error!("create_bounty failed: {err}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(BountyResponse::from(bounty)))
}

pub async fn get_bounty(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<BountyResponse>, StatusCode> {
    let bounty = sqlx::query_as::<_, Bounty>("SELECT * FROM bounties WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .map_err(|err| {
            tracing::error!("get_bounty failed: {err}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(BountyResponse::from(bounty)))
}

pub async fn update_bounty(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateBountyRequest>,
) -> Result<Json<BountyResponse>, StatusCode> {
    let bounty = sqlx::query_as::<_, Bounty>(
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
    .fetch_optional(&state.db)
    .await
    .map_err(|err| {
        tracing::error!("update_bounty failed: {err}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(BountyResponse::from(bounty)))
}

pub async fn update_bounty_status(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateBountyRequest>,
) -> Result<Json<BountyResponse>, StatusCode> {
    let status = payload.status.ok_or(StatusCode::BAD_REQUEST)?;

    let bounty = sqlx::query_as::<_, Bounty>(
        r#"
        UPDATE bounties
        SET status = $2
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(status)
    .fetch_optional(&state.db)
    .await
    .map_err(|err| {
        tracing::error!("update_bounty_status failed: {err}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(BountyResponse::from(bounty)))
}

pub async fn delete_bounty(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query("DELETE FROM bounties WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|err| {
            tracing::error!("delete_bounty failed: {err}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/bounties/{id}/sync
pub async fn sync_bounty(
    State(_state): State<AppState>,
    Path(_id): Path<Uuid>,
) -> Result<Json<BountyResponse>, StatusCode> {
    // TODO: fetch PDA via SOLANA_RPC_URL and update row
    Err(StatusCode::NOT_IMPLEMENTED)
}

/// POST /api/bounties/confirm
pub async fn confirm_bounty(
    State(_state): State<AppState>,
    Json(_payload): Json<ConfirmBountyRequest>,
) -> Result<Json<BountyResponse>, StatusCode> {
    // TODO: verify tx on RPC, upsert bounty metadata
    Err(StatusCode::NOT_IMPLEMENTED)
}
