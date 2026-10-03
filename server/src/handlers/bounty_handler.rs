use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use uuid::Uuid;

use crate::app::AppState;
use crate::auth::{can_claim_bounty, AuthUser};
use crate::dto::{
    BountyResponse, ConfirmBountyRequest, CreateBountyRequest, ListBountiesQuery,
    UpdateBountyRequest,
};
use crate::models::BountyStatus;
use crate::services::bounty as bounty_service;

/// GET /api/bounties?status=&poster=&claimer=
pub async fn list_bounties(
    State(state): State<AppState>,
    Query(query): Query<ListBountiesQuery>,
) -> Result<Json<Vec<BountyResponse>>, StatusCode> {
    let bounties = bounty_service::list(&state.db, &query)
        .await
        .map_err(StatusCode::from)?;

    Ok(Json(
        bounties.into_iter().map(BountyResponse::from).collect(),
    ))
}

pub async fn create_bounty(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(payload): Json<CreateBountyRequest>,
) -> Result<Json<BountyResponse>, StatusCode> {
    auth.ensure_wallet(&payload.poster_wallet)?;

    let bounty = bounty_service::create(&state.db, &payload)
        .await
        .map_err(StatusCode::from)?;

    Ok(Json(BountyResponse::from(bounty)))
}

pub async fn get_bounty(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<BountyResponse>, StatusCode> {
    let bounty = bounty_service::get_by_id(&state.db, id)
        .await
        .map_err(StatusCode::from)?;

    Ok(Json(BountyResponse::from(bounty)))
}

pub async fn update_bounty(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateBountyRequest>,
) -> Result<Json<BountyResponse>, StatusCode> {
    let existing = bounty_service::get_by_id(&state.db, id)
        .await
        .map_err(StatusCode::from)?;

    let is_claim_attempt = payload.claimer_wallet.is_some()
        || payload
            .status
            .is_some_and(|status| status == BountyStatus::Claimed);

    if is_claim_attempt {
        let claimer_wallet = payload
            .claimer_wallet
            .as_deref()
            .unwrap_or(auth.wallet());
        can_claim_bounty(&auth, &existing, claimer_wallet)?;

        let bounty = bounty_service::claim(&state.db, id, claimer_wallet)
            .await
            .map_err(StatusCode::from)?;

        return Ok(Json(BountyResponse::from(bounty)));
    }

    if payload.submission_uri.is_some() {
        auth.ensure_bounty_claimer(&existing)?;
    } else {
        auth.ensure_bounty_poster(&existing)?;
    }

    let bounty = bounty_service::update(&state.db, id, &payload)
        .await
        .map_err(StatusCode::from)?;

    Ok(Json(BountyResponse::from(bounty)))
}

pub async fn update_bounty_status(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateBountyRequest>,
) -> Result<Json<BountyResponse>, StatusCode> {
    let status = payload.status.ok_or(StatusCode::BAD_REQUEST)?;
    let existing = bounty_service::get_by_id(&state.db, id)
        .await
        .map_err(StatusCode::from)?;
    auth.ensure_bounty_poster(&existing)?;

    let bounty = bounty_service::update_status(&state.db, id, status)
        .await
        .map_err(StatusCode::from)?;

    Ok(Json(BountyResponse::from(bounty)))
}

pub async fn delete_bounty(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let existing = bounty_service::get_by_id(&state.db, id)
        .await
        .map_err(StatusCode::from)?;
    auth.ensure_bounty_poster(&existing)?;

    bounty_service::delete(&state.db, id)
        .await
        .map_err(StatusCode::from)?;

    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/bounties/{id}/sync
pub async fn sync_bounty(
    State(_state): State<AppState>,
    _auth: AuthUser,
    Path(_id): Path<Uuid>,
) -> Result<Json<BountyResponse>, StatusCode> {
    Err(StatusCode::NOT_IMPLEMENTED)
}

/// POST /api/bounties/confirm
pub async fn confirm_bounty(
    State(_state): State<AppState>,
    _auth: AuthUser,
    Json(_payload): Json<ConfirmBountyRequest>,
) -> Result<Json<BountyResponse>, StatusCode> {
    Err(StatusCode::NOT_IMPLEMENTED)
}
