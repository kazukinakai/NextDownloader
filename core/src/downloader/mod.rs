//! # ダウンローダーモジュール
//! 
//! ダウンロード機能の中核となるモジュールです。
//! `Downloader`トレイトと`DownloadManager`クラスを提供します。

mod h265_converter;

pub use h265_converter::H265Converter;

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
use crate::streaming::{hls::HlsDownloader, dash::DashDownloader, common::StreamingOptions};

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
    /// HLSダウンローダー
    hls_downloader: HlsDownloader,
    /// DASHダウンローダー
    dash_downloader: DashDownloader,
    /// H.265変換器
    h265_converter: Option<H265Converter>,
}

impl DownloadManager {
    /// 新しいダウンロードマネージャーを作成します
    pub fn new() -> Self {
        let h265_converter = H265Converter::new().ok();
        
        Self {
            downloads: Arc::new(Mutex::new(HashMap::new())),
            config: DownloadManagerConfig::default(),
            hls_downloader: HlsDownloader::new(),
            dash_downloader: DashDownloader::new(),
            h265_converter,
        }
    }
    
    /// 設定を指定して新しいダウンロードマネージャーを作成します
    pub fn with_config(config: DownloadManagerConfig) -> Self {
        let h265_converter = H265Converter::new().ok();
        
        Self {
            downloads: Arc::new(Mutex::new(HashMap::new())),
            config,
            hls_downloader: HlsDownloader::new(),
            dash_downloader: DashDownloader::new(),
            h265_converter,
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
        
        // コンテンツタイプに応じたダウンロード処理
        let downloads_clone = self.downloads.clone();
        let download_id_clone = download_id.clone();
        let url_clone = url.to_string();
        let destination_clone = destination.to_path_buf();
        let options_clone = options.clone();
        let h265_converter = self.h265_converter.clone();
        
        tokio::spawn(async move {
            let result = match content_type {
                ContentType::HLS => {
                    // HLSダウンロード
                    let hls_downloader = HlsDownloader::new();
                    let streaming_options = StreamingOptions {
                        output_file: destination_clone.clone(),
                        temp_dir: std::env::temp_dir().join("nextdownloader").join(&download_id_clone),
                        ..Default::default()
                    };
                    
                    // ダウンロード状態を更新
                    {
                        let mut downloads = downloads_clone.lock().unwrap();
                        if let Some(download) = downloads.get_mut(&download_id_clone) {
                            download.status = DownloadStatus::Downloading;
                            download.progress.status_message = Some("HLSストリーミングダウンロード中...".to_string());
                        }
                    }
                    
                    match hls_downloader.download_stream(&url_clone, streaming_options).await {
                        Ok(_) => {
                            // ダウンロード成功
                            let mut downloads = downloads_clone.lock().unwrap();
                            if let Some(download) = downloads.get_mut(&download_id_clone) {
                                download.status = DownloadStatus::Completed;
                                download.progress.progress = 1.0;
                                download.progress.status_message = Some("ダウンロード完了".to_string());
                                download.completed_at = Some(Utc::now().to_rfc3339());
                            }
                            
                            // H.265変換が有効な場合
                            if options_clone.convert_to_h265 && h265_converter.is_some() {
                                let converter = h265_converter.unwrap();
                                let mut downloads = downloads_clone.lock().unwrap();
                                if let Some(download) = downloads.get_mut(&download_id_clone) {
                                    download.status = DownloadStatus::Downloading;
                                    download.progress.status_message = Some("H.265に変換中...".to_string());
                                }
                                
                                // エンコーディングオプション
                                let encoding_options = options_clone.encoding_options.unwrap_or_default();
                                
                                // H.265に変換
                                match converter.convert_to_h265(
                                    &destination_clone,
                                    None,
                                    &encoding_options,
                                    None
                                ) {
                                    Ok(_) => {
                                        let mut downloads = downloads_clone.lock().unwrap();
                                        if let Some(download) = downloads.get_mut(&download_id_clone) {
                                            download.status = DownloadStatus::Completed;
                                            download.progress.status_message = Some("H.265変換完了".to_string());
                                        }
                                    },
                                    Err(e) => {
                                        error!("H.265変換エラー: {}", e);
                                        let mut downloads = downloads_clone.lock().unwrap();
                                        if let Some(download) = downloads.get_mut(&download_id_clone) {
                                            download.status = DownloadStatus::Error;
                                            download.progress.status_message = Some("H.265変換エラー".to_string());
                                            download.error_message = Some(format!("H.265変換エラー: {}", e));
                                        }
                                    }
                                }
                            }
                            
                            Ok(())
                        },
                        Err(e) => {
                            // ダウンロードエラー
                            let mut downloads = downloads_clone.lock().unwrap();
                            if let Some(download) = downloads.get_mut(&download_id_clone) {
                                download.status = DownloadStatus::Error;
                                download.progress.status_message = Some("ダウンロードエラー".to_string());
                                download.error_message = Some(format!("HLSダウンロードエラー: {}", e));
                            }
                            Err(e)
                        }
                    }
                },
                ContentType::DASH => {
                    // DASHダウンロード
                    let dash_downloader = DashDownloader::new();
                    let streaming_options = StreamingOptions {
                        output_file: destination_clone.clone(),
                        temp_dir: std::env::temp_dir().join("nextdownloader").join(&download_id_clone),
                        ..Default::default()
                    };
                    
                    // ダウンロード状態を更新
                    {
                        let mut downloads = downloads_clone.lock().unwrap();
                        if let Some(download) = downloads.get_mut(&download_id_clone) {
                            download.status = DownloadStatus::Downloading;
                            download.progress.status_message = Some("DASHストリーミングダウンロード中...".to_string());
                        }
                    }
                    
                    match dash_downloader.download_stream(&url_clone, streaming_options).await {
                        Ok(_) => {
                            // ダウンロード成功
                            let mut downloads = downloads_clone.lock().unwrap();
                            if let Some(download) = downloads.get_mut(&download_id_clone) {
                                download.status = DownloadStatus::Completed;
                                download.progress.progress = 1.0;
                                download.progress.status_message = Some("ダウンロード完了".to_string());
                                download.completed_at = Some(Utc::now().to_rfc3339());
                            }
                            
                            // H.265変換が有効な場合
                            if options_clone.convert_to_h265 && h265_converter.is_some() {
                                let converter = h265_converter.unwrap();
                                let mut downloads = downloads_clone.lock().unwrap();
                                if let Some(download) = downloads.get_mut(&download_id_clone) {
                                    download.status = DownloadStatus::Downloading;
                                    download.progress.status_message = Some("H.265に変換中...".to_string());
                                }
                                
                                // エンコーディングオプション
                                let encoding_options = options_clone.encoding_options.unwrap_or_default();
                                
                                // H.265に変換
                                match converter.convert_to_h265(
                                    &destination_clone,
                                    None,
                                    &encoding_options,
                                    None
                                ) {
                                    Ok(_) => {
                                        let mut downloads = downloads_clone.lock().unwrap();
                                        if let Some(download) = downloads.get_mut(&download_id_clone) {
                                            download.status = DownloadStatus::Completed;
                                            download.progress.status_message = Some("H.265変換完了".to_string());
                                        }
                                    },
                                    Err(e) => {
                                        error!("H.265変換エラー: {}", e);
                                        let mut downloads = downloads_clone.lock().unwrap();
                                        if let Some(download) = downloads.get_mut(&download_id_clone) {
                                            download.status = DownloadStatus::Error;
                                            download.progress.status_message = Some("H.265変換エラー".to_string());
                                            download.error_message = Some(format!("H.265変換エラー: {}", e));
                                        }
                                    }
                                }
                            }
                            
                            Ok(())
                        },
                        Err(e) => {
                            // ダウンロードエラー
                            let mut downloads = downloads_clone.lock().unwrap();
                            if let Some(download) = downloads.get_mut(&download_id_clone) {
                                download.status = DownloadStatus::Error;
                                download.progress.status_message = Some("ダウンロードエラー".to_string());
                                download.error_message = Some(format!("DASHダウンロードエラー: {}", e));
                            }
                            Err(e)
                        }
                    }
                },
                ContentType::MP4 => {
                    // 通常のHTTPダウンロード
                    Self::download_http_file(&url_clone, &destination_clone, downloads_clone.clone(), &download_id_clone).await
                },
                _ => {
                    // その他のコンテンツタイプ
                    // 実際の実装では、コンテンツタイプに応じた処理を実装する
                    // 現在はシミュレーション用のコード
                    
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
                    
                    Ok(())
                }
            };
            
            if let Err(e) = result {
                error!("ダウンロードエラー: {}", e);
            }
        });
        
        Ok(download_id)
    }
    
    /// HTTPファイルをダウンロードします
    async fn download_http_file(
        url: &str,
        destination: &Path,
        downloads: Arc<Mutex<HashMap<String, DownloadInfo>>>,
        download_id: &str,
    ) -> Result<(), anyhow::Error> {
        // HTTPクライアント
        let client = reqwest::Client::new();
        
        // ダウンロード状態を更新
        {
            let mut downloads_map = downloads.lock().unwrap();
            if let Some(download) = downloads_map.get_mut(download_id) {
                download.status = DownloadStatus::Downloading;
                download.progress.status_message = Some("HTTPダウンロード中...".to_string());
            }
        }
        
        // ヘッドリクエストでファイルサイズを取得
        let resp = client.head(url).send().await?;
        let total_size = resp.headers()
            .get(reqwest::header::CONTENT_LENGTH)
            .and_then(|val| val.to_str().ok())
            .and_then(|val| val.parse::<u64>().ok());
        
        if let Some(total) = total_size {
            let mut downloads_map = downloads.lock().unwrap();
            if let Some(download) = downloads_map.get_mut(download_id) {
                download.progress.total_size = Some(total);
            }
        }
        
        // ファイルのダウンロード
        let resp = client.get(url).send().await?;
        
        if !resp.status().is_success() {
            let mut downloads_map = downloads.lock().unwrap();
            if let Some(download) = downloads_map.get_mut(download_id) {
                download.status = DownloadStatus::Error;
                download.progress.status_message = Some("ダウンロードエラー".to_string());
                download.error_message = Some(format!("HTTPエラー: {}", resp.status()));
            }
            return Err(anyhow::anyhow!("HTTPエラー: {}", resp.status()));
        }
        
        // 親ディレクトリの作成
        if let Some(parent) = destination.parent() {
            if !parent.exists() {
                tokio::fs::create_dir_all(parent).await?;
            }
        }
        
        // ファイルを開く
        let mut file = tokio::fs::File::create(destination).await?;
        let mut stream = resp.bytes_stream();
        
        let mut downloaded = 0u64;
        let start = Instant::now();
        
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;
            tokio::io::AsyncWriteExt::write_all(&mut file, &chunk).await?;
            
            downloaded += chunk.len() as u64;
            
            let elapsed = start.elapsed().as_secs_f64();
            let progress = if let Some(total) = total_size {
                downloaded as f64 / total as f64
            } else {
                0.0
            };
            
            let speed = if elapsed > 0.0 {
                Some((downloaded as f64 / elapsed) as u64)
            } else {
                None
            };
            
            let eta = if let (Some(s), Some(t)) = (speed, total_size) {
                if s > 0 {
                    Some(((t - downloaded) / s) as u64)
                } else {
                    None
                }
            } else {
                None
            };
            
            // 進捗の更新
            let mut downloads_map = downloads.lock().unwrap();
            if let Some(download) = downloads_map.get_mut(download_id) {
                download.progress.progress = progress;
                download.progress.downloaded_size = downloaded;
                download.progress.speed = speed;
                download.progress.eta = eta;
                download.progress.status_message = Some(format!("ダウンロード中... {:.1}%", progress * 100.0));
            }
        }
        
        // ダウンロード完了
        let mut downloads_map = downloads.lock().unwrap();
        if let Some(download) = downloads_map.get_mut(download_id) {
            download.status = DownloadStatus::Completed;
            download.progress.progress = 1.0;
            download.progress.status_message = Some("ダウンロード完了".to_string());
            download.completed_at = Some(Utc::now().to_rfc3339());
        }
        
        Ok(())
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
            ffmpeg: self.h265_converter.is_some(),
        })
    }
    
    /// H.265変換器が利用可能かどうかを確認します
    pub fn is_h265_converter_available(&self) -> bool {
        self.h265_converter.is_some()
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