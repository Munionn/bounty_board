use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::dto::{CreateUserRequest, UpdateUserRequest, UserResponse};
use crate::models::User;
use crate::AppState;

pub async fn list_users(
    State(state): State<AppState>,
) -> Result<Json<Vec<UserResponse>>, StatusCode> {
    let users = sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY created_at DESC")
        .fetch_all(&state.db)
        .await
        .map_err(|err| {
            tracing::error!("list_users failed: {err}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(users.into_iter().map(UserResponse::from).collect()))
}

pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<UserResponse>, StatusCode> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .map_err(|err| {
            tracing::error!("get_user failed: {err}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(UserResponse::from(user)))
}

pub async fn get_user_by_wallet(
    State(state): State<AppState>,
    Path(wallet): Path<String>,
) -> Result<Json<UserResponse>, StatusCode> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE wallet_address = $1")
        .bind(&wallet)
        .fetch_optional(&state.db)
        .await
        .map_err(|err| {
            tracing::error!("get_user_by_wallet failed: {err}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(UserResponse::from(user)))
}

pub async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<UserResponse>, StatusCode> {
    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (wallet_address, display_name, bio)
        VALUES ($1, $2, $3)
        RETURNING *
        "#,
    )
    .bind(&payload.wallet_address)
    .bind(&payload.display_name)
    .bind(&payload.bio)
    .fetch_one(&state.db)
    .await
    .map_err(|err| {
        tracing::error!("create_user failed: {err}");
        if let sqlx::Error::Database(db_err) = &err {
            if db_err.constraint().is_some() {
                return StatusCode::CONFLICT;
            }
        }
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(UserResponse::from(user)))
}

pub async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>, StatusCode> {
    let user = sqlx::query_as::<_, User>(
        r#"
        UPDATE users
        SET
            display_name = COALESCE($2, display_name),
            bio = COALESCE($3, bio)
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(&payload.display_name)
    .bind(&payload.bio)
    .fetch_optional(&state.db)
    .await
    .map_err(|err| {
        tracing::error!("update_user failed: {err}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(UserResponse::from(user)))
}

pub async fn delete_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|err| {
            tracing::error!("delete_user failed: {err}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(StatusCode::NO_CONTENT)
}
