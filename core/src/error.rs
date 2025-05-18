//! # エラー処理モジュール
//! 
//! NextDownloaderのエラーコードと例外処理を定義します。

use std::fmt;
use std::io;
use thiserror::Error;

/// エラーコード
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    /// 無効なURL
    InvalidUrl = 1,
    /// ネットワークエラー
    NetworkError = 2,
    /// ファイルシステムエラー
    FileSystemError = 3,
    /// 依存関係エラー
    DependencyError = 4,
    /// 不明なエラー
    UnknownError = 999,
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorCode::InvalidUrl => write!(f, "InvalidUrl"),
            ErrorCode::NetworkError => write!(f, "NetworkError"),
            ErrorCode::FileSystemError => write!(f, "FileSystemError"),
            ErrorCode::DependencyError => write!(f, "DependencyError"),
            ErrorCode::UnknownError => write!(f, "UnknownError"),
        }
    }
}

/// NextDownloaderのエラー型
#[derive(Error, Debug)]
pub enum DownloaderError {
    /// 無効なURL
    #[error("無効なURL: {0}")]
    InvalidUrl(String),
    
    /// ネットワークエラー
    #[error("ネットワークエラー: {0}")]
    NetworkError(String),
    
    /// ファイルシステムエラー
    #[error("ファイルシステムエラー: {0}")]
    FileSystemError(String),
    
    /// 依存関係エラー
    #[error("依存関係エラー: {0}")]
    DependencyError(String),
    
    /// 不明なエラー
    #[error("不明なエラー: {0}")]
    UnknownError(String),
    
    /// IOエラー
    #[error("IOエラー: {0}")]
    IoError(#[from] io::Error),
    
    /// URLパースエラー
    #[error("URLパースエラー: {0}")]
    UrlParseError(#[from] url::ParseError),
    
    /// HTTPエラー
    #[error("HTTPエラー: {0}")]
    HttpError(#[from] reqwest::Error),
    
    /// JSONエラー
    #[error("JSONエラー: {0}")]
    JsonError(#[from] serde_json::Error),
}

impl DownloaderError {
    /// エラーコードを取得します
    pub fn error_code(&self) -> ErrorCode {
        match self {
            DownloaderError::InvalidUrl(_) => ErrorCode::InvalidUrl,
            DownloaderError::NetworkError(_) | DownloaderError::HttpError(_) => ErrorCode::NetworkError,
            DownloaderError::FileSystemError(_) | DownloaderError::IoError(_) => ErrorCode::FileSystemError,
            DownloaderError::DependencyError(_) => ErrorCode::DependencyError,
            DownloaderError::UnknownError(_) | DownloaderError::UrlParseError(_) | DownloaderError::JsonError(_) => ErrorCode::UnknownError,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_code_display() {
        assert_eq!(ErrorCode::InvalidUrl.to_string(), "InvalidUrl");
        assert_eq!(ErrorCode::NetworkError.to_string(), "NetworkError");
        assert_eq!(ErrorCode::FileSystemError.to_string(), "FileSystemError");
        assert_eq!(ErrorCode::DependencyError.to_string(), "DependencyError");
        assert_eq!(ErrorCode::UnknownError.to_string(), "UnknownError");
    }
    
    #[test]
    fn test_downloader_error_code() {
        assert_eq!(DownloaderError::InvalidUrl("test".to_string()).error_code(), ErrorCode::InvalidUrl);
        assert_eq!(DownloaderError::NetworkError("test".to_string()).error_code(), ErrorCode::NetworkError);
        assert_eq!(DownloaderError::FileSystemError("test".to_string()).error_code(), ErrorCode::FileSystemError);
        assert_eq!(DownloaderError::DependencyError("test".to_string()).error_code(), ErrorCode::DependencyError);
        assert_eq!(DownloaderError::UnknownError("test".to_string()).error_code(), ErrorCode::UnknownError);
    }
}