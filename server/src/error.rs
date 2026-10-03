use axum::http::StatusCode;

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("not found")]
    NotFound,
    #[error("unauthorized")]
    Unauthorized,
    #[error("forbidden")]
    Forbidden,
    #[error("conflict")]
    Conflict,
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("not implemented")]
    NotImplemented,
    #[error("internal error")]
    Internal,
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}

impl ServiceError {
    pub fn from_db(err: sqlx::Error) -> Self {
        if let sqlx::Error::Database(db_err) = &err {
            if db_err.constraint().is_some() {
                return Self::Conflict;
            }
        }
        Self::Database(err)
    }
}

impl From<ServiceError> for StatusCode {
    fn from(value: ServiceError) -> Self {
        match value {
            ServiceError::NotFound => StatusCode::NOT_FOUND,
            ServiceError::Unauthorized => StatusCode::UNAUTHORIZED,
            ServiceError::Forbidden => StatusCode::FORBIDDEN,
            ServiceError::Conflict => StatusCode::CONFLICT,
            ServiceError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ServiceError::NotImplemented => StatusCode::NOT_IMPLEMENTED,
            ServiceError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            ServiceError::Database(err) => {
                tracing::error!("database error: {err}");
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_not_found_to_404() {
        assert_eq!(StatusCode::from(ServiceError::NotFound), StatusCode::NOT_FOUND);
    }
}
