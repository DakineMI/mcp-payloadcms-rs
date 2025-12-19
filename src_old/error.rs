use rmcp::ErrorData as RpcError;

use thiserror::Error;
use tokio::io;

pub type ServiceResult<T> = core::result::Result<T, ServiceError>;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("{0}")]
    FromString(String),
    #[error("{0}")]
    RpcError(#[from] RpcError),
    #[error("{0}")]
    IoError(#[from] io::Error),
    #[error("{0}")]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("{0}")]
    Other(String),
}
