use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use nextdownloader_core::{DownloadManager, Downloader, DownloadOptions, ContentType as CoreContentType, VideoFormat as CoreVideoFormat, ErrorCode};

// UniFFIの設定
uniffi::setup_scaffolding!();

// エラー型の定義
#[derive(Debug, thiserror::Error)]
pub enum DownloaderError {
    #[error("Success")]
    Success,
    #[error("Unknown error")]
    UnknownError,
    #[error("Invalid URL")]
    InvalidUrl,
    #[error("Network error")]
    NetworkError,
    #[error("File system error")]
    FileSystemError,
    #[error("Dependency error")]
    DependencyError,
}

// コンテンツタイプの変換
impl From<CoreContentType> for ContentType {
    fn from(ct: CoreContentType) -> Self {
        match ct {
            CoreContentType::MP4 => ContentType::MP4,
            CoreContentType::HLS => ContentType::HLS,
            CoreContentType::DASH => ContentType::DASH,
            CoreContentType::YouTube => ContentType::YouTube,
            CoreContentType::Unknown => ContentType::Unknown,
        }
    }
}

// ビデオフォーマットの変換
impl From<CoreVideoFormat> for VideoFormat {
    fn from(vf: CoreVideoFormat) -> Self {
        match vf {
            CoreVideoFormat::MP4 => VideoFormat::MP4,
            CoreVideoFormat::WebM => VideoFormat::WebM,
            CoreVideoFormat::AVI => VideoFormat::AVI,
            CoreVideoFormat::MKV => VideoFormat::MKV,
            CoreVideoFormat::Unknown => VideoFormat::Unknown,
        }
    }
}

// エラーコードの変換
impl From<ErrorCode> for DownloaderError {
    fn from(ec: ErrorCode) -> Self {
        match ec {
            ErrorCode::Success => DownloaderError::Success,
            ErrorCode::InvalidUrl => DownloaderError::InvalidUrl,
            ErrorCode::NetworkError => DownloaderError::NetworkError,
            ErrorCode::FileSystemError => DownloaderError::FileSystemError,
            _ => DownloaderError::UnknownError,
        }
    }
}

// ダウンロードマネージャーの実装
pub struct DownloadManager {
    inner: Arc<Mutex<nextdownloader_core::DownloadManager>>,
    runtime: tokio::runtime::Runtime,
}

impl DownloadManager {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(nextdownloader_core::DownloadManager::new())),
            runtime: tokio::runtime::Runtime::new().unwrap(),
        }
    }
    
    pub fn check_dependencies(&self) -> Result<DependencyStatus, DownloaderError> {
        let download_manager = self.inner.lock().unwrap();
        let (ytdlp, aria2c, ffmpeg) = self.runtime.block_on(download_manager.check_dependencies());
        
        Ok(DependencyStatus {
            ytdlp,
            aria2c,
            ffmpeg,
        })
    }
    
    pub fn detect_content_type(&self, url: String) -> Result<ContentType, DownloaderError> {
        let download_manager = self.inner.lock().unwrap();
        
        // URLの検証
        if url.is_empty() {
            return Err(DownloaderError::InvalidUrl);
        }
        
        // コンテンツタイプの検出（非同期関数を同期的に実行）
        match self.runtime.block_on(download_manager.detect_content_type(&url)) {
            Ok(content_type) => Ok(content_type.into()),
            Err(e) => Err(e.into()),
        }
    }
    
    pub fn start_download(&self, url: String, destination: String, format: String) -> Result<String, DownloaderError> {
        let download_manager = self.inner.lock().unwrap();
        
        // URLとパスの検証
        if url.is_empty() {
            return Err(DownloaderError::InvalidUrl);
        }
        
        let dest_path = PathBuf::from(destination);
        
        // オプションの設定
        let mut options = DownloadOptions::default();
        if !format.is_empty() {
            options.preferred_format = Some(format);
        }
        
        // ダウンロードの開始（非同期関数を同期的に実行）
        match self.runtime.block_on(download_manager.start_download(&url, &dest_path, options)) {
            Ok(download_id) => Ok(download_id),
            Err(e) => Err(e.into()),
        }
    }
    
    pub fn get_download_progress(&self, download_id: String) -> Result<f64, DownloaderError> {
        let download_manager = self.inner.lock().unwrap();
        
        // ダウンロードIDの検証
        if download_id.is_empty() {
            return Err(DownloaderError::UnknownError);
        }
        
        // 進捗の取得
        match self.runtime.block_on(download_manager.get_download_progress(&download_id)) {
            Ok(progress) => Ok(progress),
            Err(e) => Err(e.into()),
        }
    }
    
    pub fn cancel_download(&self, download_id: String) -> Result<(), DownloaderError> {
        let download_manager = self.inner.lock().unwrap();
        
        // ダウンロードIDの検証
        if download_id.is_empty() {
            return Err(DownloaderError::UnknownError);
        }
        
        // ダウンロードのキャンセル
        match self.runtime.block_on(download_manager.cancel_download(&download_id)) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}

// デフォルト実装
impl Default for DownloadManager {
    fn default() -> Self {
        Self::new()
    }
}