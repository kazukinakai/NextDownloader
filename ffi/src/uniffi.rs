//! # UniFFI モジュール
//! 
//! NextDownloaderのUniFFIバインディングを提供します。
//! このモジュールは、Swift、Kotlin、Python、JavaScriptなどの言語から
//! NextDownloaderのコア機能を利用するためのインターフェースを提供します。

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use log::{debug, error, info, warn};

use nextdownloader_core::{
    ContentType, DownloadManager, DownloadOptions, DownloadStatus, ErrorCode, DependencyStatus,
};

use crate::DOWNLOAD_MANAGER;

/// NextDownloader APIのUniFFIインターフェース
#[uniffi::export]
pub fn initialize() {
    crate::initialize();
}

/// URLからコンテンツタイプを検出します
#[uniffi::export]
pub fn detect_content_type(url: &str) -> Result<String, String> {
    let manager = DOWNLOAD_MANAGER.lock().unwrap();
    
    match manager.detect_content_type(url) {
        Ok(content_type) => Ok(crate::content_type_to_string(content_type).to_string()),
        Err(e) => Err(e.to_string()),
    }
}

/// ダウンロードを開始します
#[uniffi::export]
pub async fn start_download(
    url: String,
    destination: String,
    format: Option<String>,
) -> Result<String, String> {
    let manager = DOWNLOAD_MANAGER.lock().unwrap();
    
    match manager.start_download(&url, &PathBuf::from(destination), format.as_deref()).await {
        Ok(download_id) => Ok(download_id),
        Err(e) => Err(e.to_string()),
    }
}

/// ダウンロードの進捗を取得します
#[uniffi::export]
pub async fn get_download_progress(download_id: String) -> Result<f64, String> {
    let manager = DOWNLOAD_MANAGER.lock().unwrap();
    
    match manager.get_download_progress(&download_id).await {
        Ok(progress) => Ok(progress),
        Err(e) => Err(e.to_string()),
    }
}

/// ダウンロードをキャンセルします
#[uniffi::export]
pub async fn cancel_download(download_id: String) -> Result<(), String> {
    let manager = DOWNLOAD_MANAGER.lock().unwrap();
    
    match manager.cancel_download(&download_id).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

/// 依存関係をチェックします
#[uniffi::export]
pub async fn check_dependencies() -> Result<DependencyStatusWrapper, String> {
    let manager = DOWNLOAD_MANAGER.lock().unwrap();
    
    match manager.check_dependencies().await {
        Ok(status) => Ok(DependencyStatusWrapper {
            ytdlp: status.ytdlp,
            aria2c: status.aria2c,
            ffmpeg: status.ffmpeg,
        }),
        Err(e) => Err(e.to_string()),
    }
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