use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::app::AppState;
use crate::auth::AuthUser;
use crate::dto::{CreateUserRequest, UpdateUserRequest, UserResponse};
use crate::services::user as user_service;

pub async fn list_users(
    State(state): State<AppState>,
) -> Result<Json<Vec<UserResponse>>, StatusCode> {
    let users = user_service::list(&state.db)
        .await
        .map_err(StatusCode::from)?;

    Ok(Json(users.into_iter().map(UserResponse::from).collect()))
}

pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<UserResponse>, StatusCode> {
    let user = user_service::get_by_id(&state.db, id)
        .await
        .map_err(StatusCode::from)?;

    Ok(Json(UserResponse::from(user)))
}

pub async fn get_user_by_wallet(
    State(state): State<AppState>,
    Path(wallet): Path<String>,
) -> Result<Json<UserResponse>, StatusCode> {
    let user = user_service::get_by_wallet(&state.db, &wallet)
        .await
        .map_err(StatusCode::from)?;

    Ok(Json(UserResponse::from(user)))
}

pub async fn create_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<UserResponse>, StatusCode> {
    auth.ensure_wallet(&payload.wallet_address)?;

    let user = user_service::create(&state.db, &payload)
        .await
        .map_err(StatusCode::from)?;

    Ok(Json(UserResponse::from(user)))
}

pub async fn update_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>, StatusCode> {
    let existing = user_service::get_by_id(&state.db, id)
        .await
        .map_err(StatusCode::from)?;
    auth.ensure_wallet(&existing.wallet_address)?;

    let user = user_service::update(&state.db, id, &payload)
        .await
        .map_err(StatusCode::from)?;

    Ok(Json(UserResponse::from(user)))
}

pub async fn delete_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let existing = user_service::get_by_id(&state.db, id)
        .await
        .map_err(StatusCode::from)?;
    auth.ensure_wallet(&existing.wallet_address)?;

    user_service::delete(&state.db, id)
        .await
        .map_err(StatusCode::from)?;

    Ok(StatusCode::NO_CONTENT)
}
