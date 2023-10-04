use thiserror::Error;

#[derive(Debug, Error)]
pub enum IaError {
    #[error("IO error: {}", _0)]
    IoError(#[from] std::io::Error),
    #[error("Storage error: {}", _0)]
    Sled(#[from] sled::Error),
    #[error("Message: {}", _0)]
    Message(String),
    #[error("Invalid UTF-8: {}", _0)]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error("Store deadlock: {}", _0)]
    StoreDeadLock(&'static str),
    #[error("Bincode: {}", _0)]
    Bincode(#[from] bincode::Error),
    #[error("OpenAI API: {}", _0)]
    OpenAIError(#[from] api_connector::openai::OpenAIError),
    #[error("Guy: {}", _0)]
    Guy(#[from] guy::error::GuyError),
}

pub type IaResult<T> = std::result::Result<T, IaError>;
