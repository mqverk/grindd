use thiserror::Error;

#[derive(Debug, Error)]
pub enum GrinddError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid command: {0}")]
    InvalidCommand(String),
}

pub type Result<T> = std::result::Result<T, GrinddError>;