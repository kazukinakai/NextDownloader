//! # UniFFI モジュール
//! 
//! NextDownloaderのUniFFIバインディングを提供します。
//! このモジュールは、Swift、Kotlin、Python、JavaScriptなどの言語から
//! NextDownloaderのコア機能を利用するためのインターフェースを提供します。

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use futures_util::future::TryFutureExt;

use log::{debug, error, info, warn};

use nextdownloader_core::{
    ContentType, DownloadManager, DownloadStatus, ErrorCode, DependencyStatus,
};
use nextdownloader_core::error::DownloaderError as CoreDownloaderError;

use crate::DOWNLOAD_MANAGER;

/// UniFFI用のエラー型
#[derive(Debug, thiserror::Error)]
#[uniffi::export(with_foreign = "DownloaderError")]
pub enum DownloaderError {
    #[error("Invalid URL: {0}")]
    InvalidURL(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("File system error: {0}")]
    FileSystemError(String),
    
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
    
    #[error("Dependency error: {0}")]
    DependencyError(String),
    
    #[error("Unknown error: {0}")]
    UnknownError(String),
}

/// コアエラーからUniFFIエラーへの変換関数
impl From<CoreDownloaderError> for DownloaderError {
    fn from(err: CoreDownloaderError) -> Self {
        match err.code() {
            ErrorCode::InvalidUrl => DownloaderError::InvalidURL(err.to_string()),
            ErrorCode::NetworkError => DownloaderError::NetworkError(err.to_string()),
            ErrorCode::FileSystemError => DownloaderError::FileSystemError(err.to_string()),
            ErrorCode::UnsupportedFormat => DownloaderError::UnsupportedFormat(err.to_string()),
            ErrorCode::DependencyError => DownloaderError::DependencyError(err.to_string()),
            _ => DownloaderError::UnknownError(err.to_string()),
        }
    }
}

/// NextDownloader APIのUniFFIインターフェース
#[uniffi::export]
pub fn initialize() {
    crate::initialize();
}

/// URLからコンテンツタイプを検出します
#[uniffi::export]
pub fn detect_content_type(url: &str) -> Result<String, DownloaderError> {
    let manager = DOWNLOAD_MANAGER.lock().unwrap();
    
    match manager.detect_content_type(url) {
        Ok(content_type) => Ok(crate::content_type_to_string(content_type).to_string()),
        Err(e) => Err(e.into()),
    }
}

/// ダウンロードを開始します
#[uniffi::export]
pub fn start_download(
    url: String,
    destination: String,
    format: Option<String>,
) -> Result<String, DownloaderError> {
    let manager = DOWNLOAD_MANAGER.lock().unwrap();
    let url_ref = url.as_str();
    let dest_path = PathBuf::from(destination);
    let format_ref = format.as_deref();
    
    // tokioのランタイムを作成して非同期処理を実行
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        manager.start_download(url_ref, &dest_path, format_ref).await.map_err(Into::into)
    })
}

/// ダウンロードの進捗を取得します
#[uniffi::export]
pub fn get_download_progress(download_id: String) -> Result<f64, DownloaderError> {
    let manager = DOWNLOAD_MANAGER.lock().unwrap();
    let download_id_ref = download_id.as_str();
    
    // tokioのランタイムを作成して非同期処理を実行
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        manager.get_download_progress(download_id_ref).await.map_err(Into::into)
    })
}

/// ダウンロードをキャンセルします
#[uniffi::export]
pub fn cancel_download(download_id: String) -> Result<(), DownloaderError> {
    let manager = DOWNLOAD_MANAGER.lock().unwrap();
    let download_id_ref = download_id.as_str();
    
    // tokioのランタイムを作成して非同期処理を実行
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        manager.cancel_download(download_id_ref).await.map_err(Into::into)
    })
}

/// 依存関係をチェックします
#[uniffi::export]
pub fn check_dependencies() -> Result<DependencyStatusWrapper, DownloaderError> {
    let manager = DOWNLOAD_MANAGER.lock().unwrap();
    
    // tokioのランタイムを作成して非同期処理を実行
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        manager.check_dependencies().await.map(|status| {
            DependencyStatusWrapper {
                ytdlp: status.ytdlp,
                aria2c: status.aria2c,
                ffmpeg: status.ffmpeg,
            }
        }).map_err(Into::into)
    })
}

/// NextDownloaderのバージョンを取得します
#[uniffi::export]
pub fn get_version() -> String {
    nextdownloader_core::version().to_string()
}

/// 依存関係のステータスラッパー
#[uniffi::export]
pub struct DependencyStatusWrapper {
    pub ytdlp: bool,
    pub aria2c: bool,
    pub ffmpeg: bool,
}

uniffi::include_scaffolding!("nextdownloader");