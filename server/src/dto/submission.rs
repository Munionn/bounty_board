use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
pub struct CreateSubmissionRequest {
    pub submitter_wallet: String,
    pub submission_uri: String,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SubmissionResponse {
    pub id: Uuid,
    pub bounty_id: Uuid,
    pub submitter_wallet: String,
    pub submission_uri: String,
    pub note: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<crate::models::Submission> for SubmissionResponse {
    fn from(s: crate::models::Submission) -> Self {
        Self {
            id: s.id,
            bounty_id: s.bounty_id,
            submitter_wallet: s.submitter_wallet,
            submission_uri: s.submission_uri,
            note: s.note,
            created_at: s.created_at,
        }
    }
}
