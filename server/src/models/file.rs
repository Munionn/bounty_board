use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Type;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "file_status", rename_all = "snake_case")]
pub enum FileStatus {
    Pending,
    Processing,
    Uploaded,
    Failed,
    Deleted,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct ChatFile {
    pub id: Uuid,
    pub owner_wallet: String,
    pub bucket: String,
    pub object_key: String,
    pub object_version_id: Option<String>,
    pub original_filename: String,
    pub content_type: Option<String>,
    pub size_bytes: Option<i64>,
    pub checksum_sha256: Option<String>,
    pub status: FileStatus,
    pub bounty_id: Option<Uuid>,
    pub chat_id: Option<Uuid>,
    pub preview_object_key: Option<String>,
    pub extracted_text: Option<String>,
    pub created_at: DateTime<Utc>,
    pub uploaded_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}
