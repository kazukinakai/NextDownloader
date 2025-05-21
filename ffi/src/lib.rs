//! # NextDownloader FFI
//! 
//! NextDownloaderのFFIレイヤーです。
//! C FFIとUniFFIの両方のバインディングを提供します。

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_double, c_int};
use std::path::PathBuf;
use std::ptr;
use std::sync::{Arc, Mutex, Once};

use lazy_static::lazy_static;
use log::{debug, error, info, warn};

use nextdownloader_core::{
    ContentType, DownloadManager, DownloadOptions, DownloadStatus, ErrorCode,
};

// モジュールのエクスポート
#[cfg(feature = "c-ffi")]
pub mod c_ffi;
#[cfg(feature = "uniffi")]
pub mod uniffi;
pub mod flutter_bridge;

// グローバルダウンロードマネージャーのインスタンス
lazy_static! {
    static ref DOWNLOAD_MANAGER: Arc<Mutex<DownloadManager>> = Arc::new(Mutex::new(DownloadManager::new()));
    static ref INIT: Once = Once::new();
}

/// FFIレイヤーを初期化します
pub fn initialize() {
    INIT.call_once(|| {
        // ロギングの初期化
        env_logger::init();
        info!("NextDownloader FFIレイヤーを初期化しています...");
    });
}

/// エラーコードを文字列に変換します
pub fn error_code_to_string(code: ErrorCode) -> &'static str {
    match code {
        ErrorCode::InvalidUrl => "InvalidUrl",
        ErrorCode::NetworkError => "NetworkError",
        ErrorCode::FileSystemError => "FileSystemError",
        ErrorCode::DependencyError => "DependencyError",
        ErrorCode::UnknownError => "UnknownError",
    }
}

/// コンテンツタイプを文字列に変換します
pub fn content_type_to_string(content_type: ContentType) -> &'static str {
    match content_type {
        ContentType::MP4 => "MP4",
        ContentType::HLS => "HLS",
        ContentType::DASH => "DASH",
        ContentType::YouTube => "YouTube",
        ContentType::Unknown => "Unknown",
    }
}

/// 文字列からコンテンツタイプに変換します
pub fn string_to_content_type(s: &str) -> ContentType {
    match s.to_lowercase().as_str() {
        "mp4" => ContentType::MP4,
        "hls" => ContentType::HLS,
        "dash" => ContentType::DASH,
        "youtube" => ContentType::YouTube,
        _ => ContentType::Unknown,
    }
}

/// ダウンロードステータスを文字列に変換します
pub fn download_status_to_string(status: DownloadStatus) -> &'static str {
    match status {
        DownloadStatus::Initializing => "Initializing",
        DownloadStatus::Downloading => "Downloading",
        DownloadStatus::Paused => "Paused",
        DownloadStatus::Completed => "Completed",
        DownloadStatus::Error => "Error",
        DownloadStatus::Cancelled => "Cancelled",
    }
}

/// 文字列からダウンロードステータスに変換します
pub fn string_to_download_status(s: &str) -> DownloadStatus {
    match s.to_lowercase().as_str() {
        "initializing" => DownloadStatus::Initializing,
        "downloading" => DownloadStatus::Downloading,
        "paused" => DownloadStatus::Paused,
        "completed" => DownloadStatus::Completed,
        "error" => DownloadStatus::Error,
        "cancelled" => DownloadStatus::Cancelled,
        _ => DownloadStatus::Error,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_content_type_conversion() {
        assert_eq!(content_type_to_string(ContentType::MP4), "MP4");
        assert_eq!(string_to_content_type("mp4"), ContentType::MP4);
        assert_eq!(string_to_content_type("unknown"), ContentType::Unknown);
    }
    
    #[test]
    fn test_download_status_conversion() {
        assert_eq!(download_status_to_string(DownloadStatus::Downloading), "Downloading");
        assert_eq!(string_to_download_status("downloading"), DownloadStatus::Downloading);
        assert_eq!(string_to_download_status("unknown"), DownloadStatus::Error);
    }
}