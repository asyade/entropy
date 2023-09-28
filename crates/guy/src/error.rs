use thiserror::Error;

#[derive(Debug, Error)]
pub enum GuyError {
    #[error("IO error: {}", _0)]
    IoError(#[from] std::io::Error),
    #[error("YAML error: {}", _0)]
    YamlError(#[from] serde_yaml::Error),
}

pub type Result<T> = std::result::Result<T, GuyError>;
