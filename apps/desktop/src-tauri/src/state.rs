//! # アプリケーション状態管理
//! 
//! Tauriアプリケーションの状態を管理するモジュールです。

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use nextdownloader_core::{
    AppConfig, DownloadManager, DownloadProgress, DownloadStatus,
};

/// アプリケーションの状態
#[derive(Debug)]
pub struct AppState {
    /// ダウンロードマネージャー
    pub download_manager: Arc<Mutex<DownloadManager>>,
    /// アプリケーション設定
    pub config: Arc<RwLock<AppConfig>>,
    /// アクティブなダウンロード
    pub downloads: Arc<RwLock<HashMap<String, DownloadInfo>>>,
}

/// ダウンロード情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadInfo {
    /// ダウンロードID
    pub id: String,
    /// ダウンロードURL
    pub url: String,
    /// 保存先パス
    pub destination: String,
    /// ファイル名
    pub filename: String,
    /// コンテンツタイプ
    pub content_type: String,
    /// ダウンロードの進捗状況
    pub progress: f64,
    /// ダウンロード速度（バイト/秒）
    pub speed: Option<u64>,
    /// 推定残り時間（秒）
    pub eta: Option<u64>,
    /// ダウンロード済みサイズ（バイト）
    pub downloaded_size: u64,
    /// 合計サイズ（バイト）
    pub total_size: Option<u64>,
    /// ステータスメッセージ
    pub status_message: Option<String>,
    /// ダウンロードステータス
    pub status: String,
    /// 作成日時
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 更新日時
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl AppState {
    /// 新しいアプリケーション状態を作成します
    pub fn new() -> Self {
        // 設定ファイルのパスを取得
        let config_path = AppConfig::default_config_path();
        
        // 設定を読み込む
        let config = match AppConfig::load(&config_path) {
            Ok(config) => config,
            Err(e) => {
                error!("設定の読み込みに失敗しました: {}", e);
                AppConfig::default()
            }
        };
        
        // ダウンロードマネージャーを作成
        let download_manager = DownloadManager::with_config(config.download_manager.clone());
        
        Self {
            download_manager: Arc::new(Mutex::new(download_manager)),
            config: Arc::new(RwLock::new(config)),
            downloads: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// ダウンロード情報を追加します
    pub async fn add_download(&self, id: String, url: String, destination: String, content_type: String) {
        let filename = PathBuf::from(&destination)
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown.file".to_string());
        
        let now = chrono::Utc::now();
        
        let download_info = DownloadInfo {
            id: id.clone(),
            url,
            destination,
            filename,
            content_type,
            progress: 0.0,
            speed: None,
            eta: None,
            downloaded_size: 0,
            total_size: None,
            status_message: Some("初期化中...".to_string()),
            status: "initializing".to_string(),
            created_at: now,
            updated_at: now,
        };
        
        let mut downloads = self.downloads.write().await;
        downloads.insert(id, download_info);
    }
    
    /// ダウンロード情報を更新します
    pub async fn update_download(&self, id: &str, progress: DownloadProgress) {
        let mut downloads = self.downloads.write().await;
        
        if let Some(download) = downloads.get_mut(id) {
            download.progress = progress.progress;
            download.speed = progress.speed;
            download.eta = progress.eta;
            download.downloaded_size = progress.downloaded_size;
            download.total_size = progress.total_size;
            download.status_message = progress.status_message;
            download.status = progress.status.to_string().to_lowercase();
            download.updated_at = chrono::Utc::now();
        }
    }
    
    /// ダウンロード情報を取得します
    pub async fn get_downloads(&self) -> Vec<DownloadInfo> {
        let downloads = self.downloads.read().await;
        downloads.values().cloned().collect()
    }
    
    /// ダウンロード情報を削除します
    pub async fn remove_download(&self, id: &str) {
        let mut downloads = self.downloads.write().await;
        downloads.remove(id);
    }
}