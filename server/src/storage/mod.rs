use std::time::Duration;

use aws_sdk_s3::config::{BehaviorVersion, Credentials, Region};
use aws_sdk_s3::error::SdkError;
use aws_sdk_s3::operation::{
    delete_object::DeleteObjectError, get_object::GetObjectError, head_object::HeadObjectError,
    put_object::PutObjectError,
};
use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client as S3Client;
use chrono::{DateTime, Utc};
use thiserror::Error;

#[derive(Clone)]
pub struct StorageState {
    pub client: S3Client,
    pub default_bucket: String,
    pub public_endpoint: String,
}

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("S3 get-object request failed: {0}")]
    GetObject(#[from] SdkError<GetObjectError>),
    #[error("S3 head-object request failed: {0}")]
    HeadObject(#[from] SdkError<HeadObjectError>),
    #[error("S3 put-object request failed: {0}")]
    PutObject(#[from] SdkError<PutObjectError>),
    #[error("S3 delete-object request failed: {0}")]
    DeleteObject(#[from] SdkError<DeleteObjectError>),
    #[error("failed to read object body: {0}")]
    Body(String),
    #[error("invalid storage input: {0}")]
    InvalidInput(String),
}

impl StorageState {
    pub fn new(client: S3Client, default_bucket: String, public_endpoint: String) -> Self {
        StorageState {
            client,
            default_bucket,
            public_endpoint,
        }
    }

    pub fn from_env() -> Self {
        let endpoint = std::env::var("MINIO_ENDPOINT").expect("MINIO_ENDPOINT must be set in .env");
        let public_endpoint =
            std::env::var("MINIO_PUBLIC_ENDPOINT").unwrap_or_else(|_| endpoint.clone());
        let access_key =
            std::env::var("MINIO_ACCESS_KEY").expect("MINIO_ACCESS_KEY must be set in .env");
        let secret_key =
            std::env::var("MINIO_SECRET_KEY").expect("MINIO_SECRET_KEY must be set in .env");
        let region = std::env::var("MINIO_REGION").unwrap_or_else(|_| "us-east-1".into());
        let default_bucket =
            std::env::var("MINIO_BUCKET").expect("MINIO_BUCKET must be set in .env");

        let credentials = Credentials::new(access_key, secret_key, None, None, "minio");

        let config = aws_sdk_s3::Config::builder()
            .behavior_version(BehaviorVersion::latest())
            .credentials_provider(credentials)
            .endpoint_url(&endpoint)
            .region(Region::new(region))
            .force_path_style(true)
            .build();

        let client = S3Client::from_conf(config);

        Self::new(client, default_bucket, public_endpoint)
    }

    pub fn submission_object_key(bounty_id: &str, filename: &str) -> String {
        format!("bounties/{bounty_id}/{filename}")
    }

    pub fn chat_object_key(bounty_id: &str, file_id: &str, filename: &str) -> String {
        format!("chat/{bounty_id}/{file_id}/{filename}")
    }

    pub fn rewrite_presigned_url_for_browser(&self, url: String) -> String {
        if self.public_endpoint.is_empty() {
            return url;
        }
        let internal = std::env::var("MINIO_ENDPOINT").unwrap_or_default();
        if internal.is_empty() || internal == self.public_endpoint {
            return url;
        }
        url.replace(&internal, &self.public_endpoint)
    }

    pub async fn presigned_put_url(
        &self,
        object_key: &str,
        content_type: Option<&str>,
        expires_secs: u64,
    ) -> Result<(String, DateTime<Utc>), StorageError> {
        if object_key.trim().is_empty() {
            return Err(StorageError::InvalidInput(
                "object key cannot be empty".into(),
            ));
        }

        let presigning_config = PresigningConfig::expires_in(Duration::from_secs(expires_secs))
            .map_err(|err| StorageError::InvalidInput(err.to_string()))?;

        let mut request = self
            .client
            .put_object()
            .bucket(&self.default_bucket)
            .key(object_key);

        if let Some(content_type) = content_type {
            request = request.content_type(content_type);
        }

        let presigned = request
            .presigned(presigning_config)
            .await
            .map_err(|err| StorageError::InvalidInput(err.to_string()))?;

        let expires_at = Utc::now() + Duration::from_secs(expires_secs);
        Ok((
            self.rewrite_presigned_url_for_browser(presigned.uri().to_string()),
            expires_at,
        ))
    }

    pub async fn presigned_get_url(
        &self,
        object_key: &str,
        expires_secs: u64,
    ) -> Result<(String, DateTime<Utc>), StorageError> {
        if object_key.trim().is_empty() {
            return Err(StorageError::InvalidInput(
                "object key cannot be empty".into(),
            ));
        }

        let presigning_config = PresigningConfig::expires_in(Duration::from_secs(expires_secs))
            .map_err(|err| StorageError::InvalidInput(err.to_string()))?;

        let presigned = self
            .client
            .get_object()
            .bucket(&self.default_bucket)
            .key(object_key)
            .presigned(presigning_config)
            .await
            .map_err(|err| StorageError::InvalidInput(err.to_string()))?;

        let expires_at = Utc::now() + Duration::from_secs(expires_secs);
        Ok((
            self.rewrite_presigned_url_for_browser(presigned.uri().to_string()),
            expires_at,
        ))
    }

    pub async fn head_object(
        &self,
        object_key: &str,
    ) -> Result<(Option<i64>, Option<String>), StorageError> {
        if object_key.trim().is_empty() {
            return Err(StorageError::InvalidInput(
                "object key cannot be empty".into(),
            ));
        }

        let response = self
            .client
            .head_object()
            .bucket(&self.default_bucket)
            .key(object_key)
            .send()
            .await?;

        Ok((response.content_length(), response.content_type().map(str::to_string)))
    }

    pub async fn get_object(&self, key: &str) -> Result<Vec<u8>, StorageError> {
        if key.trim().is_empty() {
            return Err(StorageError::InvalidInput(
                "object key cannot be empty".into(),
            ));
        }

        let object = self
            .client
            .get_object()
            .bucket(&self.default_bucket)
            .key(key)
            .send()
            .await?;

        let data = object
            .body
            .collect()
            .await
            .map_err(|e| StorageError::Body(e.to_string()))?
            .into_bytes()
            .to_vec();

        Ok(data)
    }

    pub async fn upload_object(
        &self,
        bounty_id: &str,
        filename: &str,
        body: &[u8],
        content_type: Option<&str>,
    ) -> Result<String, StorageError> {
        if bounty_id.trim().is_empty() {
            return Err(StorageError::InvalidInput(
                "bounty_id cannot be empty".into(),
            ));
        }
        if filename.trim().is_empty() {
            return Err(StorageError::InvalidInput(
                "filename cannot be empty".into(),
            ));
        }
        if body.is_empty() {
            return Err(StorageError::InvalidInput(
                "upload body cannot be empty".into(),
            ));
        }

        let object_key = Self::submission_object_key(bounty_id, filename);

        let mut request = self
            .client
            .put_object()
            .bucket(&self.default_bucket)
            .key(&object_key)
            .body(ByteStream::from(body.to_vec()));

        if let Some(content_type) = content_type {
            request = request.content_type(content_type);
        }

        request.send().await?;

        Ok(object_key)
    }

    pub async fn put_object_bytes(
        &self,
        object_key: &str,
        body: &[u8],
        content_type: Option<&str>,
    ) -> Result<(), StorageError> {
        let mut request = self
            .client
            .put_object()
            .bucket(&self.default_bucket)
            .key(object_key)
            .body(ByteStream::from(body.to_vec()));

        if let Some(content_type) = content_type {
            request = request.content_type(content_type);
        }

        request.send().await?;
        Ok(())
    }

    pub async fn delete_object(
        &self,
        bounty_id: &str,
        file_name: &str,
    ) -> Result<String, StorageError> {
        if bounty_id.trim().is_empty() {
            return Err(StorageError::InvalidInput(
                "bounty input couldnt been empty".into(),
            ));
        }
        if file_name.trim().is_empty() {
            return Err(StorageError::InvalidInput(
                "file name coudn't been empty".into(),
            ));
        }
        let object_key = Self::submission_object_key(bounty_id, file_name);
        self.client
            .delete_object()
            .bucket(&self.default_bucket)
            .key(&object_key)
            .send()
            .await?;

        Ok(object_key)
    }
}
