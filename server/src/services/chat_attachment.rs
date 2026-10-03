use sqlx::PgPool;
use uuid::Uuid;

use crate::app::{AppState, ChatHub};
use crate::dto::PublishAttachmentRequest;
use crate::dto::{
    AttachmentResponse, CompleteUploadRequest, FileDownloadResponse, FileStatusResponse,
    MessageResponse, RegisterFileRequest, RegisterFileResponse, SendMessagePayload, ServerWsEvent,
    WsMessageAckPayload, WsMessageCreatedPayload,
};
use crate::error::ServiceError;
use crate::models::{Bounty, ChatFile, FileStatus};
use crate::services::{attachment_processor, chat as chat_service};
use crate::storage::StorageState;

const PRESIGN_UPLOAD_SECS: u64 = 900;
const PRESIGN_DOWNLOAD_SECS: u64 = 900;
const DEFAULT_MAX_FILE_BYTES: i64 = 10 * 1024 * 1024;

pub fn max_file_bytes() -> i64 {
    std::env::var("CHAT_MAX_FILE_BYTES")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(DEFAULT_MAX_FILE_BYTES)
}

pub fn sanitize_filename(name: &str) -> Result<String, ServiceError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(ServiceError::BadRequest(
            "filename cannot be empty".into(),
        ));
    }

    let base = trimmed
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(trimmed)
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_'))
        .collect::<String>();

    if base.is_empty() {
        return Err(ServiceError::BadRequest("invalid filename".into()));
    }

    Ok(base.chars().take(180).collect())
}

pub fn validate_content_type(content_type: &Option<String>) -> Result<(), ServiceError> {
    let Some(content_type) = content_type.as_deref() else {
        return Err(ServiceError::BadRequest(
            "content_type is required for chat attachments".into(),
        ));
    };

    if is_allowed_content_type(content_type) {
        Ok(())
    } else {
        Err(ServiceError::BadRequest(format!(
            "content type {content_type} is not allowed"
        )))
    }
}

fn is_allowed_content_type(content_type: &str) -> bool {
    content_type.starts_with("image/")
        || content_type.starts_with("text/")
        || matches!(
            content_type,
            "application/pdf"
                | "application/json"
                | "application/zip"
                | "application/x-zip-compressed"
                | "application/octet-stream"
        )
}

pub async fn register_attachment(
    pool: &PgPool,
    storage: &StorageState,
    bounty: &Bounty,
    chat_id: Uuid,
    owner_wallet: &str,
    payload: &RegisterFileRequest,
) -> Result<RegisterFileResponse, ServiceError> {
    chat_service::ensure_participant(bounty, owner_wallet)?;

    if payload.size_bytes <= 0 {
        return Err(ServiceError::BadRequest(
            "size_bytes must be greater than zero".into(),
        ));
    }
    if payload.size_bytes > max_file_bytes() {
        return Err(ServiceError::BadRequest(format!(
            "file exceeds max size of {} bytes",
            max_file_bytes()
        )));
    }

    validate_content_type(&payload.content_type)?;
    let safe_filename = sanitize_filename(&payload.original_filename)?;
    let file_id = Uuid::new_v4();
    let object_key = StorageState::chat_object_key(
        &bounty.id.to_string(),
        &file_id.to_string(),
        &safe_filename,
    );

    let file = sqlx::query_as::<_, ChatFile>(
        r#"
        INSERT INTO files (
            id,
            owner_wallet,
            bucket,
            object_key,
            original_filename,
            content_type,
            size_bytes,
            status,
            bounty_id,
            chat_id
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, 'pending', $8, $9)
        RETURNING *
        "#,
    )
    .bind(file_id)
    .bind(owner_wallet)
    .bind(&storage.default_bucket)
    .bind(&object_key)
    .bind(&safe_filename)
    .bind(&payload.content_type)
    .bind(payload.size_bytes)
    .bind(bounty.id)
    .bind(chat_id)
    .fetch_one(pool)
    .await
    .map_err(ServiceError::from_db)?;

    let (upload_url, expires_at) = storage
        .presigned_put_url(
            &file.object_key,
            file.content_type.as_deref(),
            PRESIGN_UPLOAD_SECS,
        )
        .await
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    Ok(RegisterFileResponse {
        file_id: file.id,
        status: file_status_label(file.status),
        upload_url,
        upload_expires_at: expires_at,
        object_key: file.object_key,
    })
}

pub async fn complete_upload(
    state: AppState,
    bounty: &Bounty,
    file_id: Uuid,
    owner_wallet: &str,
    payload: CompleteUploadRequest,
) -> Result<FileStatusResponse, ServiceError> {
    chat_service::ensure_participant(bounty, owner_wallet)?;

    let file = load_owned_file(&state.db, bounty.id, file_id, owner_wallet).await?;
    if !matches!(file.status, FileStatus::Pending | FileStatus::Failed) {
        return Err(ServiceError::Conflict);
    }

    state
        .storage
        .head_object(&file.object_key)
        .await
        .map_err(|_| {
            ServiceError::BadRequest(
                "uploaded object not found; finish the browser upload first".into(),
            )
        })?;

    let file = sqlx::query_as::<_, ChatFile>(
        r#"
        UPDATE files
        SET status = 'processing'
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(file_id)
    .fetch_one(&state.db)
    .await
    .map_err(ServiceError::from_db)?;

    let chat = chat_service::ensure_chat_for_bounty(&state.db, bounty.id).await?;
    let publish = payload.publish;
    let owner = owner_wallet.to_string();
    let bounty_id = bounty.id;

    tokio::spawn(async move {
        if let Err(err) =
            run_processing_job(state, bounty_id, chat.id, file_id, owner, publish).await
        {
            tracing::error!(file_id = %file_id, error = %err, "attachment processing job failed");
        }
    });

    Ok(to_status_response(&file))
}

async fn run_processing_job(
    state: AppState,
    bounty_id: Uuid,
    chat_id: Uuid,
    file_id: Uuid,
    owner_wallet: String,
    publish: Option<PublishAttachmentRequest>,
) -> Result<(), ServiceError> {
    let processed = attachment_processor::process_attachment(&state.db, &state.storage, file_id).await?;
    if processed.status != FileStatus::Uploaded {
        return Ok(());
    }

    let Some(publish) = publish else {
        return Ok(());
    };

    let client_message_id = publish.client_message_id.clone();
    let message = chat_service::send_message(
        &state.db,
        chat_id,
        bounty_id,
        &owner_wallet,
        &SendMessagePayload {
            body: publish.body,
            file_ids: vec![file_id],
            client_message_id: publish.client_message_id,
        },
    )
    .await?;

    broadcast_message(&state.chat_hub, chat_id, message, client_message_id);
    Ok(())
}

fn broadcast_message(
    chat_hub: &ChatHub,
    chat_id: Uuid,
    message: MessageResponse,
    client_message_id: Option<String>,
) {
    if let Some(client_message_id) = client_message_id {
        let ack = ServerWsEvent::MessageAck(WsMessageAckPayload {
            client_message_id: Some(client_message_id),
            message: message.clone(),
        });
        if let Ok(json) = serde_json::to_string(&ack) {
            let hub = chat_hub.clone();
            tokio::spawn(async move {
                hub.publish(chat_id, json).await;
            });
        }
    }

    let created = ServerWsEvent::MessageCreated(WsMessageCreatedPayload { message });
    if let Ok(json) = serde_json::to_string(&created) {
        let hub = chat_hub.clone();
        tokio::spawn(async move {
            hub.publish(chat_id, json).await;
        });
    }
}

pub async fn get_file_status(
    pool: &PgPool,
    bounty: &Bounty,
    file_id: Uuid,
    wallet: &str,
) -> Result<FileStatusResponse, ServiceError> {
    chat_service::ensure_participant(bounty, wallet)?;
    let file = load_participant_file(pool, bounty.id, file_id).await?;
    Ok(to_status_response(&file))
}

pub async fn get_download_url(
    pool: &PgPool,
    storage: &StorageState,
    bounty: &Bounty,
    file_id: Uuid,
    wallet: &str,
    preview: bool,
) -> Result<FileDownloadResponse, ServiceError> {
    chat_service::ensure_participant(bounty, wallet)?;
    let file = load_participant_file(pool, bounty.id, file_id).await?;

    if file.status != FileStatus::Uploaded {
        return Err(ServiceError::Conflict);
    }

    let object_key = if preview {
        file.preview_object_key
            .as_deref()
            .unwrap_or(&file.object_key)
    } else {
        &file.object_key
    };

    let (url, expires_at) = storage
        .presigned_get_url(object_key, PRESIGN_DOWNLOAD_SECS)
        .await
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    Ok(FileDownloadResponse {
        url,
        expires_at,
        file_id: file.id,
        original_filename: file.original_filename,
        content_type: file.content_type,
        size_bytes: file.size_bytes,
        preview: preview && file.preview_object_key.is_some(),
    })
}

pub async fn load_attachment_metadata(
    pool: &PgPool,
    message_id: Uuid,
) -> Result<Vec<AttachmentResponse>, ServiceError> {
    let rows = sqlx::query_as::<_, AttachmentRow>(
        r#"
        SELECT
            f.id AS file_id,
            f.original_filename,
            f.content_type,
            f.size_bytes,
            mf.position,
            mf.caption,
            f.extracted_text,
            f.preview_object_key,
            f.status AS file_status
        FROM message_files mf
        INNER JOIN files f ON f.id = mf.file_id
        WHERE mf.message_id = $1
        ORDER BY mf.position ASC
        "#,
    )
    .bind(message_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| AttachmentResponse {
            file_id: row.file_id,
            original_filename: row.original_filename,
            content_type: row.content_type,
            size_bytes: row.size_bytes,
            position: row.position,
            caption: row.caption,
            status: file_status_label(row.file_status),
            extracted_text_preview: row.extracted_text.as_deref().map(truncate_preview),
            has_preview: row.preview_object_key.is_some(),
        })
        .collect())
}

async fn load_owned_file(
    pool: &PgPool,
    bounty_id: Uuid,
    file_id: Uuid,
    owner_wallet: &str,
) -> Result<ChatFile, ServiceError> {
    sqlx::query_as::<_, ChatFile>(
        r#"
        SELECT * FROM files
        WHERE id = $1 AND bounty_id = $2 AND owner_wallet = $3
        "#,
    )
    .bind(file_id)
    .bind(bounty_id)
    .bind(owner_wallet)
    .fetch_optional(pool)
    .await?
    .ok_or(ServiceError::NotFound)
}

async fn load_participant_file(
    pool: &PgPool,
    bounty_id: Uuid,
    file_id: Uuid,
) -> Result<ChatFile, ServiceError> {
    sqlx::query_as::<_, ChatFile>(
        "SELECT * FROM files WHERE id = $1 AND bounty_id = $2",
    )
    .bind(file_id)
    .bind(bounty_id)
    .fetch_optional(pool)
    .await?
    .ok_or(ServiceError::NotFound)
}

fn to_status_response(file: &ChatFile) -> FileStatusResponse {
    FileStatusResponse {
        file_id: file.id,
        status: file_status_label(file.status),
        original_filename: file.original_filename.clone(),
        content_type: file.content_type.clone(),
        size_bytes: file.size_bytes,
        extracted_text_preview: file.extracted_text.as_deref().map(truncate_preview),
        has_preview: file.preview_object_key.is_some(),
        uploaded_at: file.uploaded_at,
    }
}

fn file_status_label(status: FileStatus) -> String {
    match status {
        FileStatus::Pending => "pending".into(),
        FileStatus::Processing => "processing".into(),
        FileStatus::Uploaded => "ready".into(),
        FileStatus::Failed => "failed".into(),
        FileStatus::Deleted => "deleted".into(),
    }
}

fn truncate_preview(value: &str) -> String {
    value.chars().take(240).collect()
}

#[derive(Debug, sqlx::FromRow)]
struct AttachmentRow {
    file_id: Uuid,
    original_filename: String,
    content_type: Option<String>,
    size_bytes: Option<i64>,
    position: i16,
    caption: Option<String>,
    extracted_text: Option<String>,
    preview_object_key: Option<String>,
    file_status: FileStatus,
}
