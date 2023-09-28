use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiConnectorError {
    #[error("IO error: {}", _0)]
    IoError(#[from] std::io::Error),
    #[error("JSON error: {}", _0)]
    JsonError(#[from] serde_json::Error),
    #[error("reqwest error: {}", _0)]
    HttpClientError(#[from] reqwest::Error),
}

pub type Result<T> = std::result::Result<T, ApiConnectorError>;
