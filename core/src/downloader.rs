//! # ダウンローダーモジュール
//! 
//! ダウンロード機能の中核となるモジュールです。
//! `Downloader`トレイトと`DownloadManager`クラスを提供します。

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use chrono::Utc;
use log::{debug, error, info, warn};
use tokio::sync::mpsc;
use tokio::time;
use uuid::Uuid;

use crate::content_type::ContentType;
use crate::error::{DownloaderError, ErrorCode};
use crate::{DownloadInfo, DownloadProgress, DownloadStatus, DownloadManagerConfig};
use crate::encoding::{EncodingManager, EncodingOptions, VideoFormat as EncodingFormat};

/// 動画フォーマット
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoFormat {
    /// MP4
    MP4,
    /// WebM
    WebM,
    /// MKV
    MKV,
    /// 自動選択（最高品質）
    Best,
}

/// ダウンロードオプション
#[derive(Debug, Clone)]
pub struct DownloadOptions {
    /// 保存先パス
    pub destination: PathBuf,
    /// 希望するフォーマット（動画の場合）
    pub format: Option<VideoFormat>,
    /// 音声のみをダウンロードするか
    pub audio_only: bool,
    /// 字幕をダウンロードするか
    pub download_subtitles: bool,
    /// 最大解像度（動画の場合）
    pub max_resolution: Option<String>,
    /// 最大ファイルサイズ（バイト単位）
    pub max_file_size: Option<u64>,
    /// 接続タイムアウト（秒）
    pub timeout: Option<u64>,
    /// 再試行回数
    pub retry_count: Option<u32>,
    /// H.265に変換するかどうか
    pub convert_to_h265: bool,
    /// 変換オプション
    pub encoding_options: Option<EncodingOptions>,
}

impl Default for DownloadOptions {
    fn default() -> Self {
        Self {
            destination: PathBuf::from("./downloads"),
            format: None,
            audio_only: false,
            download_subtitles: false,
            max_resolution: None,
            max_file_size: None,
            timeout: Some(30),
            retry_count: Some(3),
            convert_to_h265: false,
            encoding_options: None,
        }
    }
}

/// ダウンローダートレイト
/// 
/// 様々なダウンロード方法を実装するためのトレイトです。
pub trait Downloader: Send + Sync {
    /// ダウンロードを開始します
    fn start(&self, url: &str, options: &DownloadOptions) -> Result<String, DownloaderError>;
    
    /// ダウンロードの進捗を取得します
    fn get_progress(&self, id: &str) -> Result<DownloadProgress, DownloaderError>;
    
    /// ダウンロードを一時停止します
    fn pause(&self, id: &str) -> Result<(), DownloaderError>;
    
    /// ダウンロードを再開します
    fn resume(&self, id: &str) -> Result<(), DownloaderError>;
    
    /// ダウンロードをキャンセルします
    fn cancel(&self, id: &str) -> Result<(), DownloaderError>;
    
    /// ダウンロードの情報を取得します
    fn get_info(&self, id: &str) -> Result<DownloadInfo, DownloaderError>;
    
    /// ダウンロードが完了したかどうかを確認します
    fn is_completed(&self, id: &str) -> Result<bool, DownloaderError>;
}

/// ダウンロードマネージャー
/// 
/// 複数のダウンロードを管理するクラスです。
pub struct DownloadManager {
    /// ダウンロード情報のマップ
    downloads: Arc<Mutex<HashMap<String, DownloadInfo>>>,
    /// 設定
    config: DownloadManagerConfig,
}

impl DownloadManager {
    /// 新しいダウンロードマネージャーを作成します
    pub fn new() -> Self {
        Self {
            downloads: Arc::new(Mutex::new(HashMap::new())),
            config: DownloadManagerConfig::default(),
        }
    }
    
    /// 設定を指定して新しいダウンロードマネージャーを作成します
    pub fn with_config(config: DownloadManagerConfig) -> Self {
        Self {
            downloads: Arc::new(Mutex::new(HashMap::new())),
            config,
        }
    }
    
    /// URLのコンテンツタイプを検出します
    pub fn detect_content_type(&self, url: &str) -> Result<ContentType, DownloaderError> {
        // URLの検証
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(DownloaderError::InvalidUrl(format!("無効なURL: {}", url)));
        }
        
        // コンテンツタイプの検出
        Ok(ContentType::detect_from_url(url))
    }
    
    /// ダウンロードを開始します
    pub async fn start_download(
        &self,
        url: &str,
        destination: &Path,
        format: Option<&str>,
        convert_to_h265: bool,
    ) -> Result<String, DownloaderError> {
        // URLの検証
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(DownloaderError::InvalidUrl(format!("無効なURL: {}", url)));
        }
        
        // コンテンツタイプの検出
        let content_type = ContentType::detect_from_url(url);
        
        // ダウンロードIDの生成
        let download_id = Uuid::new_v4().to_string();
        
        // ダウンロードオプションの作成
        let mut options = DownloadOptions::default();
        options.destination = destination.to_path_buf();
        options.convert_to_h265 = convert_to_h265;
        
        // H.265変換が指定されている場合、デフォルトのエンコードオプションを設定
        if convert_to_h265 {
            options.encoding_options = Some(EncodingOptions::default());
        }
        
        // フォーマットの設定（指定されている場合）
        if let Some(fmt) = format {
            match fmt.to_lowercase().as_str() {
                "mp4" => options.format = Some(VideoFormat::MP4),
                "webm" => options.format = Some(VideoFormat::WebM),
                "mkv" => options.format = Some(VideoFormat::MKV),
                "best" => options.format = Some(VideoFormat::Best),
                _ => {}
            }
        }
        
        // ダウンロード情報の作成
        let download_info = DownloadInfo {
            id: download_id.clone(),
            url: url.to_string(),
            destination: destination.to_path_buf(),
            content_type,
            status: DownloadStatus::Initializing,
            progress: DownloadProgress {
                id: download_id.clone(),
                progress: 0.0,
                speed: None,
                eta: None,
                downloaded_size: 0,
                total_size: None,
                status_message: Some("初期化中...".to_string()),
            },
            created_at: Utc::now().to_rfc3339(),
            completed_at: None,
            error_message: None,
        };
        
        // ダウンロード情報をマップに追加
        {
            let mut downloads = self.downloads.lock().unwrap();
            downloads.insert(download_id.clone(), download_info);
        }
        
        // 実際のダウンロード処理はここで実装
        // 現在はモック実装のため、実際のダウンロードは行わない
        
        // 進捗更新のシミュレーション（実際の実装では削除）
        let downloads_clone = self.downloads.clone();
        let download_id_clone = download_id.clone();
        
        tokio::spawn(async move {
            // 進捗更新のシミュレーション
            for i in 1..=10 {
                time::sleep(Duration::from_secs(1)).await;
                
                let progress = i as f64 / 10.0;
                let mut downloads = downloads_clone.lock().unwrap();
                
                if let Some(download) = downloads.get_mut(&download_id_clone) {
                    download.progress.progress = progress;
                    download.progress.downloaded_size = (progress * 1_000_000.0) as u64;
                    download.progress.total_size = Some(1_000_000);
                    download.progress.speed = Some((progress * 100_000.0) as u64);
                    download.progress.eta = Some((10 - i) as u64);
                    download.progress.status_message = Some(format!("ダウンロード中... {}%", (progress * 100.0) as u32));
                    
                    if progress >= 1.0 {
                        download.status = DownloadStatus::Completed;
                        download.completed_at = Some(Utc::now().to_rfc3339());
                        download.progress.status_message = Some("ダウンロード完了".to_string());
                    } else {
                        download.status = DownloadStatus::Downloading;
                    }
                }
            }
        });
        
        Ok(download_id)
    }
    
    /// ダウンロードの進捗を取得します
    pub async fn get_download_progress(&self, download_id: &str) -> Result<f64, DownloaderError> {
        let downloads = self.downloads.lock().unwrap();
        
        if let Some(download) = downloads.get(download_id) {
            Ok(download.progress.progress)
        } else {
            Err(DownloaderError::UnknownError(format!("ダウンロードIDが見つかりません: {}", download_id)))
        }
    }
    
    /// ダウンロードをキャンセルします
    pub async fn cancel_download(&self, download_id: &str) -> Result<(), DownloaderError> {
        let mut downloads = self.downloads.lock().unwrap();
        
        if let Some(download) = downloads.get_mut(download_id) {
            download.status = DownloadStatus::Cancelled;
            download.progress.status_message = Some("キャンセルされました".to_string());
            Ok(())
        } else {
            Err(DownloaderError::UnknownError(format!("ダウンロードIDが見つかりません: {}", download_id)))
        }
    }
    
    /// すべてのダウンロード情報を取得します
    pub fn get_all_downloads(&self) -> Vec<DownloadInfo> {
        let downloads = self.downloads.lock().unwrap();
        downloads.values().cloned().collect()
    }
    
    /// 依存関係をチェックします
    pub async fn check_dependencies(&self) -> Result<crate::DependencyStatus, DownloaderError> {
        // 実際の実装では、コマンドの存在をチェックする
        // 現在はモック実装
        Ok(crate::DependencyStatus {
            ytdlp: true,
            aria2c: true,
            ffmpeg: true,
        })
    }
}

impl Default for DownloadManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_detect_content_type() {
        let manager = DownloadManager::new();
        assert_eq!(manager.detect_content_type("https://www.youtube.com/watch?v=dQw4w9WgXcQ").unwrap(), ContentType::YouTube);
        assert_eq!(manager.detect_content_type("https://example.com/video.mp4").unwrap(), ContentType::MP4);
        assert!(manager.detect_content_type("invalid-url").is_err());
    }
}