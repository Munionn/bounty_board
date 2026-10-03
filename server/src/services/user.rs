use sqlx::PgPool;
use uuid::Uuid;

use crate::dto::{CreateUserRequest, UpdateUserRequest};
use crate::error::ServiceError;
use crate::models::User;

pub async fn list(pool: &PgPool) -> Result<Vec<User>, ServiceError> {
    let users = sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY created_at DESC")
        .fetch_all(pool)
        .await?;
    Ok(users)
}

pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<User, ServiceError> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(ServiceError::NotFound)
}

pub async fn get_by_wallet(pool: &PgPool, wallet: &str) -> Result<User, ServiceError> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE wallet_address = $1")
        .bind(wallet)
        .fetch_optional(pool)
        .await?
        .ok_or(ServiceError::NotFound)
}

pub async fn create(pool: &PgPool, payload: &CreateUserRequest) -> Result<User, ServiceError> {
    sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (wallet_address, display_name, bio)
        VALUES ($1, $2, $3)
        RETURNING *
        "#,
    )
    .bind(&payload.wallet_address)
    .bind(&payload.display_name)
    .bind(&payload.bio)
    .fetch_one(pool)
    .await
    .map_err(ServiceError::from_db)
}

pub async fn update(
    pool: &PgPool,
    id: Uuid,
    payload: &UpdateUserRequest,
) -> Result<User, ServiceError> {
    sqlx::query_as::<_, User>(
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
    .fetch_optional(pool)
    .await?
    .ok_or(ServiceError::NotFound)
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), ServiceError> {
    let result = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(ServiceError::NotFound);
    }

    Ok(())
}

pub async fn upsert_by_wallet(pool: &PgPool, wallet: &str) -> Result<User, ServiceError> {
    if let Ok(user) = get_by_wallet(pool, wallet).await {
        return Ok(user);
    }

    sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (wallet_address)
        VALUES ($1)
        RETURNING *
        "#,
    )
    .bind(wallet)
    .fetch_one(pool)
    .await
    .map_err(ServiceError::from_db)
}
