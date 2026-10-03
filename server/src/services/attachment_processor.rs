use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::ServiceError;
use crate::models::{ChatFile, FileStatus};
use crate::storage::StorageState;

const TEXT_PREVIEW_LIMIT: usize = 4_096;

pub async fn process_attachment(
    pool: &PgPool,
    storage: &StorageState,
    file_id: Uuid,
) -> Result<ChatFile, ServiceError> {
    let file = load_file(pool, file_id).await?;

    if file.status != FileStatus::Processing {
        return Ok(file);
    }

    let (size, head_type) = storage
        .head_object(&file.object_key)
        .await
        .map_err(|err| ServiceError::BadRequest(format!("uploaded object missing: {err}")))?;

    if size.unwrap_or(0) <= 0 {
        return mark_failed(pool, file_id, "uploaded object is empty").await;
    }

    if let Some(expected) = file.size_bytes {
        if size.unwrap_or(0) != expected {
            return mark_failed(
                pool,
                file_id,
                "uploaded object size does not match declared size",
            )
            .await;
        }
    }

    let bytes = storage
        .get_object(&file.object_key)
        .await
        .map_err(|err| ServiceError::BadRequest(format!("failed to read uploaded object: {err}")))?;

    if !is_allowed_content(&file.content_type, &bytes) {
        return mark_failed(pool, file_id, "file type is not allowed for chat attachments").await;
    }

    let checksum = hex_encode_sha256(&bytes);
    let extracted_text = extract_text_preview(&file.content_type, &bytes);
    let preview_object_key = maybe_store_text_preview(storage, &file, extracted_text.as_deref()).await?;

    sqlx::query_as::<_, ChatFile>(
        r#"
        UPDATE files
        SET
            status = 'uploaded',
            uploaded_at = NOW(),
            checksum_sha256 = $2,
            extracted_text = $3,
            preview_object_key = COALESCE($4, preview_object_key),
            content_type = COALESCE($5, content_type),
            size_bytes = COALESCE($6, size_bytes)
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(file_id)
    .bind(checksum)
    .bind(extracted_text)
    .bind(preview_object_key)
    .bind(head_type)
    .bind(size)
    .fetch_one(pool)
    .await
    .map_err(ServiceError::from_db)
}

async fn load_file(pool: &PgPool, file_id: Uuid) -> Result<ChatFile, ServiceError> {
    sqlx::query_as::<_, ChatFile>("SELECT * FROM files WHERE id = $1")
        .bind(file_id)
        .fetch_optional(pool)
        .await?
        .ok_or(ServiceError::NotFound)
}

async fn mark_failed(pool: &PgPool, file_id: Uuid, reason: &str) -> Result<ChatFile, ServiceError> {
    tracing::warn!(file_id = %file_id, reason, "attachment processing failed");

    sqlx::query_as::<_, ChatFile>(
        r#"
        UPDATE files
        SET status = 'failed', extracted_text = $2
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(file_id)
    .bind(reason)
    .fetch_one(pool)
    .await
    .map_err(ServiceError::from_db)
}

fn hex_encode_sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

fn is_allowed_content(content_type: &Option<String>, bytes: &[u8]) -> bool {
    if bytes.starts_with(b"MZ") || bytes.starts_with(b"\x7fELF") {
        return false;
    }

    let Some(content_type) = content_type.as_deref() else {
        return false;
    };

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

fn extract_text_preview(content_type: &Option<String>, bytes: &[u8]) -> Option<String> {
    let content_type = content_type.as_deref()?;
    let is_text = content_type.starts_with("text/")
        || matches!(content_type, "application/json" | "application/xml");

    if !is_text {
        return None;
    }

    let text = String::from_utf8_lossy(bytes);
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }

    Some(
        trimmed
            .chars()
            .take(TEXT_PREVIEW_LIMIT)
            .collect::<String>(),
    )
}

async fn maybe_store_text_preview(
    storage: &StorageState,
    file: &ChatFile,
    extracted_text: Option<&str>,
) -> Result<Option<String>, ServiceError> {
    let Some(text) = extracted_text.filter(|value| !value.is_empty()) else {
        return Ok(None);
    };

    let preview_key = format!("{}.preview.txt", file.object_key);
    storage
        .put_object_bytes(&preview_key, text.as_bytes(), Some("text/plain; charset=utf-8"))
        .await
        .map_err(|err| ServiceError::BadRequest(format!("failed to store preview: {err}")))?;

    Ok(Some(preview_key))
}
