use thiserror::Error;

#[derive(Error, Debug)]
pub enum DownloadError {
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("File error: {0}")]
    FileError(String),
    
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

pub type Result<T> = std::result::Result<T, DownloadError>;