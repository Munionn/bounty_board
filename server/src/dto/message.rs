use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Shared message payload (text + optional already-uploaded file ids)
// ---------------------------------------------------------------------------

/// Text and attachment ids for a chat message.
/// Files must be uploaded over HTTP first; only pass `file_ids` here.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SendMessagePayload {
    #[serde(default)]
    pub body: String,
    #[serde(default)]
    pub file_ids: Vec<Uuid>,
    /// Client-generated id so the UI can match an optimistic row with the server ack.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_message_id: Option<String>,
}

// ---------------------------------------------------------------------------
// WebSocket: client → server
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientWsEvent {
    /// Send a chat message (after any files were uploaded via HTTP).
    SendMessage {
        #[serde(flatten)]
        payload: SendMessagePayload,
    },
    /// Keep-alive; server replies with `ServerWsEvent::Pong`.
    Ping,
}

/// JWT passed as query param because browser WebSocket APIs cannot set headers reliably.
#[derive(Debug, Clone, Deserialize)]
pub struct WsAuthQuery {
    pub token: String,
}

// ---------------------------------------------------------------------------
// WebSocket: server → client
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerWsEvent {
    /// Sent once after the socket is authenticated and joined to the bounty chat.
    Connected(WsConnectedPayload),
    /// Recent messages for initial render (optional snapshot on connect).
    History(WsHistoryPayload),
    /// Broadcast when anyone posts a message (including the sender).
    MessageCreated(WsMessageCreatedPayload),
    /// Ack only to the sender; includes `client_message_id` when provided.
    MessageAck(WsMessageAckPayload),
    /// Keep-alive response.
    Pong,
    /// Validation/auth/storage failure for a client action.
    Error(WsErrorPayload),
}

#[derive(Debug, Clone, Serialize)]
pub struct WsConnectedPayload {
    pub chat_id: Uuid,
    pub bounty_id: Uuid,
}

#[derive(Debug, Clone, Serialize)]
pub struct WsHistoryPayload {
    pub messages: Vec<MessageResponse>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WsMessageCreatedPayload {
    pub message: MessageResponse,
}

#[derive(Debug, Clone, Serialize)]
pub struct WsMessageAckPayload {
    pub client_message_id: Option<String>,
    pub message: MessageResponse,
}

#[derive(Debug, Clone, Serialize)]
pub struct WsErrorPayload {
    pub code: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_message_id: Option<String>,
}

// ---------------------------------------------------------------------------
// HTTP: file upload (keep on REST — do not send bytes over WebSocket)
// ---------------------------------------------------------------------------

/// Body for `POST /api/bounties/{id}/chat/files`
#[derive(Debug, Clone, Deserialize)]
pub struct RegisterFileRequest {
    pub original_filename: String,
    pub content_type: Option<String>,
    pub size_bytes: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct RegisterFileResponse {
    pub file_id: Uuid,
    pub status: String,
    pub upload_url: String,
    pub upload_expires_at: DateTime<Utc>,
    pub object_key: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CompleteUploadRequest {
    #[serde(default)]
    pub publish: Option<PublishAttachmentRequest>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PublishAttachmentRequest {
    #[serde(default)]
    pub body: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_message_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FileStatusResponse {
    pub file_id: Uuid,
    pub status: String,
    pub original_filename: String,
    pub content_type: Option<String>,
    pub size_bytes: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extracted_text_preview: Option<String>,
    pub has_preview: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uploaded_at: Option<DateTime<Utc>>,
}

/// Response for `GET /api/bounties/{id}/chat/files/{file_id}/download`
#[derive(Debug, Clone, Serialize)]
pub struct FileDownloadResponse {
    pub url: String,
    pub expires_at: DateTime<Utc>,
    pub file_id: Uuid,
    pub original_filename: String,
    pub content_type: Option<String>,
    pub size_bytes: Option<i64>,
    pub preview: bool,
}

// ---------------------------------------------------------------------------
// HTTP: optional history fetch (or use `ServerWsEvent::History` on connect)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct ListMessagesQuery {
    #[serde(default = "default_message_limit")]
    pub limit: i64,
    /// Return messages created strictly before this timestamp (cursor pagination).
    pub before: Option<DateTime<Utc>>,
}

fn default_message_limit() -> i64 {
    50
}

// ---------------------------------------------------------------------------
// Shared message shape (history, broadcasts, acks)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct AttachmentResponse {
    pub file_id: Uuid,
    pub original_filename: String,
    pub content_type: Option<String>,
    pub size_bytes: Option<i64>,
    pub position: i16,
    pub caption: Option<String>,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extracted_text_preview: Option<String>,
    pub has_preview: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct MessageResponse {
    pub id: Uuid,
    pub bounty_id: Uuid,
    pub chat_id: Uuid,
    pub sender_wallet: String,
    pub body: String,
    pub attachments: Vec<AttachmentResponse>,
    pub created_at: DateTime<Utc>,
}
