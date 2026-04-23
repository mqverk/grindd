use thiserror::Error;

#[derive(Debug, Error)]
pub enum GrinddError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("config error: {0}")]
    Config(String),
    #[error("unsupported on this platform: {0}")]
    Unsupported(String),
    #[error("runtime error: {0}")]
    Runtime(String),
    #[error("cgroup error: {0}")]
    Cgroup(String),
    #[error("image error: {0}")]
    Image(String),
    #[error("network error: {0}")]
    Network(String),
    #[error("daemon error: {0}")]
    Daemon(String),
    #[error("build error: {0}")]
    Build(String),
}

pub type Result<T> = std::result::Result<T, GrinddError>;
