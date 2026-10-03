use sqlx::PgPool;
use uuid::Uuid;

use crate::dto::{AttachmentResponse, ListMessagesQuery, MessageResponse, SendMessagePayload};
use crate::error::ServiceError;
use crate::models::{Bounty, BountyStatus, Chat, Message};

/// Only the employer (poster) and assigned consumer (claimer) may use bounty chat.
pub fn ensure_participant(bounty: &Bounty, wallet: &str) -> Result<(), ServiceError> {
    if !matches!(bounty.status, BountyStatus::Claimed | BountyStatus::Closed) {
        return Err(ServiceError::Forbidden);
    }

    let Some(claimer) = bounty.claimer_wallet.as_deref() else {
        return Err(ServiceError::Forbidden);
    };

    if wallet == bounty.poster_wallet || wallet == claimer {
        return Ok(());
    }

    Err(ServiceError::Forbidden)
}

pub async fn ensure_chat_for_bounty(pool: &PgPool, bounty_id: Uuid) -> Result<Chat, ServiceError> {
    if let Some(chat) = sqlx::query_as::<_, Chat>("SELECT * FROM chats WHERE bounty_id = $1")
        .bind(bounty_id)
        .fetch_optional(pool)
        .await?
    {
        return Ok(chat);
    }

    sqlx::query_as::<_, Chat>(
        r#"
        INSERT INTO chats (bounty_id)
        VALUES ($1)
        RETURNING *
        "#,
    )
    .bind(bounty_id)
    .fetch_one(pool)
    .await
    .map_err(ServiceError::from_db)
}

pub async fn list_messages(
    pool: &PgPool,
    chat_id: Uuid,
    bounty_id: Uuid,
    query: &ListMessagesQuery,
) -> Result<Vec<MessageResponse>, ServiceError> {
    let limit = query.limit.clamp(1, 100);

    let rows = sqlx::query_as::<_, Message>(
        r#"
        SELECT * FROM messages
        WHERE chat_id = $1
          AND ($2::timestamptz IS NULL OR created_at < $2)
        ORDER BY created_at DESC
        LIMIT $3
        "#,
    )
    .bind(chat_id)
    .bind(query.before)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    let mut messages = Vec::with_capacity(rows.len());
    for row in rows {
        messages.push(to_message_response(pool, bounty_id, row).await?);
    }

    messages.reverse();
    Ok(messages)
}

pub async fn send_message(
    pool: &PgPool,
    chat_id: Uuid,
    bounty_id: Uuid,
    sender_wallet: &str,
    payload: &SendMessagePayload,
) -> Result<MessageResponse, ServiceError> {
    if payload.body.trim().is_empty() && payload.file_ids.is_empty() {
        return Err(ServiceError::BadRequest(
            "message must include body text or file_ids".into(),
        ));
    }

    let message = sqlx::query_as::<_, Message>(
        r#"
        INSERT INTO messages (chat_id, sender_wallet, body)
        VALUES ($1, $2, $3)
        RETURNING *
        "#,
    )
    .bind(chat_id)
    .bind(sender_wallet)
    .bind(payload.body.trim())
    .fetch_one(pool)
    .await
    .map_err(ServiceError::from_db)?;

    for (position, file_id) in payload.file_ids.iter().enumerate() {
        attach_file(pool, message.id, *file_id, sender_wallet, position as i16).await?;
    }

    to_message_response(pool, bounty_id, message).await
}

async fn attach_file(
    pool: &PgPool,
    message_id: Uuid,
    file_id: Uuid,
    owner_wallet: &str,
    position: i16,
) -> Result<(), ServiceError> {
    let attached = sqlx::query(
        r#"
        INSERT INTO message_files (message_id, file_id, position)
        SELECT $1, f.id, $3
        FROM files f
        WHERE f.id = $2
          AND f.owner_wallet = $4
          AND f.status = 'uploaded'
        "#,
    )
    .bind(message_id)
    .bind(file_id)
    .bind(position)
    .bind(owner_wallet)
    .execute(pool)
    .await?;

    if attached.rows_affected() == 0 {
        return Err(ServiceError::BadRequest(format!(
            "file {file_id} is missing, not uploaded, or not owned by sender"
        )));
    }

    Ok(())
}

async fn load_attachments(pool: &PgPool, message_id: Uuid) -> Result<Vec<AttachmentResponse>, ServiceError> {
    super::chat_attachment::load_attachment_metadata(pool, message_id).await
}

async fn to_message_response(
    pool: &PgPool,
    bounty_id: Uuid,
    message: Message,
) -> Result<MessageResponse, ServiceError> {
    let attachments = load_attachments(pool, message.id).await?;

    Ok(MessageResponse {
        id: message.id,
        bounty_id,
        chat_id: message.chat_id,
        sender_wallet: message.sender_wallet,
        body: message.body,
        attachments,
        created_at: message.created_at,
    })
}

