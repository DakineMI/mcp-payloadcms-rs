use std::io;

use rmcp::service::ServiceError as RpcServiceError;
use thiserror::Error;

pub type ServiceResult<T> = Result<T, ServiceError>;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("{0}")]
    FromString(String),
    #[error("{0}")]
    RpcError(#[from] RpcServiceError),
    #[error("{0}")]
    IoError(#[from] io::Error),
    #[error("{0}")]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("API error: {0}")]
    ApiError(String),
    #[error("{0}")]
    Other(String),
}
