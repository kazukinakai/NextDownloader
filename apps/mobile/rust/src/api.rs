use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use lazy_static::lazy_static;

// グローバルダウンロードマネージャーのインスタンス
lazy_static! {
    static ref DOWNLOAD_MANAGER: Arc<Mutex<DownloadManager>> = Arc::new(Mutex::new(DownloadManager::new()));
}

/// ダウンロードステータス
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DownloadStatus {
    Pending,
    Downloading,
    Completed,
    Failed,
    Cancelled,
    Paused,
}

/// ダウンロードタスクの情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadTask {
    /// タスクID
    pub id: String,
    /// ダウンロードURL
    pub url: String,
    /// ファイル名
    pub file_name: String,
    /// 進捗率（0.0〜1.0）
    pub progress: f32,
    /// ダウンロードステータス
    pub status: DownloadStatus,
    /// ステータスメッセージ
    pub status_message: Option<String>,
    /// ファイルサイズ（バイト）
    pub file_size: Option<u64>,
    /// ダウンロード速度（バイト/秒）
    pub download_speed: Option<u64>,
    /// 残り時間（秒）
    pub eta: Option<u64>,
    /// 作成日時（ISO 8601形式）
    pub created_at: String,
    /// 更新日時（ISO 8601形式）
    pub updated_at: String,
}

/// 動画フォーマット
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum VideoFormat {
    MP4,
    WebM,
    MKV,
    Best,
}

/// ダウンロードオプション
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadOptions {
    /// 保存先ディレクトリ
    pub destination: String,
    /// 動画フォーマット
    pub format: Option<VideoFormat>,
    /// 音声のみをダウンロードするかどうか
    pub audio_only: bool,
    /// 字幕をダウンロードするかどうか
    pub download_subtitles: bool,
    /// 最大解像度
    pub max_resolution: Option<String>,
    /// 最大ファイルサイズ（バイト）
    pub max_file_size: Option<u64>,
    /// タイムアウト（秒）
    pub timeout: Option<u32>,
    /// リトライ回数
    pub retry_count: Option<u32>,
}

/// 依存関係の状態
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyStatus {
    /// yt-dlpがインストールされているかどうか
    pub ytdlp: bool,
    /// aria2cがインストールされているかどうか
    pub aria2c: bool,
    /// ffmpegがインストールされているかどうか
    pub ffmpeg: bool,
}

/// ダウンロードマネージャー
/// 
/// NextDownloaderのコア機能を提供します
pub struct DownloadManager {
    // 実際の実装では、NextDownloaderコアのDownloadManagerを使用
    // ここではモック実装
    downloads: Arc<Mutex<Vec<DownloadTask>>>,
}

impl DownloadManager {
    /// 新しいダウンロードマネージャーを作成
    pub fn new() -> Self {
        // モックデータを作成
        let now = chrono::Utc::now().to_rfc3339();
        let mock_downloads = vec![
            DownloadTask {
                id: "task_1".to_string(),
                url: "https://example.com/file1.zip".to_string(),
                file_name: "file1.zip".to_string(),
                progress: 0.75,
                status: DownloadStatus::Downloading,
                status_message: Some("ダウンロード中...".to_string()),
                file_size: Some(1024 * 1024 * 10), // 10MB
                download_speed: Some(1024 * 1024), // 1MB/s
                eta: Some(10), // 10秒
                created_at: now.clone(),
                updated_at: now.clone(),
            },
            DownloadTask {
                id: "task_2".to_string(),
                url: "https://example.com/file2.mp4".to_string(),
                file_name: "file2.mp4".to_string(),
                progress: 1.0,
                status: DownloadStatus::Completed,
                status_message: Some("完了".to_string()),
                file_size: Some(1024 * 1024 * 100), // 100MB
                download_speed: None,
                eta: None,
                created_at: now.clone(),
                updated_at: now.clone(),
            },
        ];
        
        Self {
            downloads: Arc::new(Mutex::new(mock_downloads)),
        }
    }

    /// 新しいダウンロードを開始する
    pub fn start_download(&self, url: &str, options: &DownloadOptions) -> Result<String> {
        // 実際の実装では、NextDownloaderコアを使用してダウンロードを開始
        // ここではモック実装
        let now = chrono::Utc::now().to_rfc3339();
        let id = uuid::Uuid::new_v4().to_string();
        
        let file_name = if let Some(idx) = url.rfind('/') {
            url[idx + 1..].to_string()
        } else {
            format!("download_{}", id)
        };
        
        let task = DownloadTask {
            id: id.clone(),
            url: url.to_string(),
            file_name,
            progress: 0.0,
            status: DownloadStatus::Pending,
            status_message: Some("準備中...".to_string()),
            file_size: None,
            download_speed: None,
            eta: None,
            created_at: now.clone(),
            updated_at: now,
        };
        
        let mut downloads = self.downloads.lock().unwrap();
        downloads.push(task);
        
        Ok(id)
    }

    /// ダウンロードの進捗を取得する
    pub fn get_download_progress(&self, download_id: &str) -> Result<f32> {
        // 実際の実装では、ダウンロードマネージャーから進捗を取得
        // ここではモック実装
        let downloads = self.downloads.lock().unwrap();
        if let Some(task) = downloads.iter().find(|t| t.id == download_id) {
            Ok(task.progress)
        } else {
            Err(anyhow::anyhow!("ダウンロードIDが見つかりません: {}", download_id))
        }
    }

    /// ダウンロードをキャンセルする
    pub fn cancel_download(&self, download_id: &str) -> Result<()> {
        // 実際の実装では、ダウンロードマネージャーにキャンセル要求を送信
        // ここではモック実装
        let mut downloads = self.downloads.lock().unwrap();
        if let Some(task) = downloads.iter_mut().find(|t| t.id == download_id) {
            task.status = DownloadStatus::Cancelled;
            task.status_message = Some("キャンセルされました".to_string());
            task.updated_at = chrono::Utc::now().to_rfc3339();
            Ok(())
        } else {
            Err(anyhow::anyhow!("ダウンロードIDが見つかりません: {}", download_id))
        }
    }

    /// ダウンロードを一時停止する
    pub fn pause_download(&self, download_id: &str) -> Result<()> {
        // 実際の実装では、ダウンロードマネージャーに一時停止要求を送信
        // ここではモック実装
        let mut downloads = self.downloads.lock().unwrap();
        if let Some(task) = downloads.iter_mut().find(|t| t.id == download_id) {
            task.status = DownloadStatus::Paused;
            task.status_message = Some("一時停止中".to_string());
            task.updated_at = chrono::Utc::now().to_rfc3339();
            Ok(())
        } else {
            Err(anyhow::anyhow!("ダウンロードIDが見つかりません: {}", download_id))
        }
    }

    /// ダウンロードを再開する
    pub fn resume_download(&self, download_id: &str) -> Result<()> {
        // 実際の実装では、ダウンロードマネージャーに再開要求を送信
        // ここではモック実装
        let mut downloads = self.downloads.lock().unwrap();
        if let Some(task) = downloads.iter_mut().find(|t| t.id == download_id) {
            task.status = DownloadStatus::Downloading;
            task.status_message = Some("ダウンロード中...".to_string());
            task.updated_at = chrono::Utc::now().to_rfc3339();
            Ok(())
        } else {
            Err(anyhow::anyhow!("ダウンロードIDが見つかりません: {}", download_id))
        }
    }

    /// すべてのダウンロードタスクを取得する
    pub fn get_all_downloads(&self) -> Vec<DownloadTask> {
        // 実際の実装では、ダウンロードマネージャーからすべてのタスクを取得
        // ここではモック実装
        let downloads = self.downloads.lock().unwrap();
        downloads.clone()
    }

    /// 依存関係をチェックする
    pub fn check_dependencies(&self) -> Result<DependencyStatus> {
        // 実際の実装では、実際に依存関係をチェック
        // ここではモック実装
        Ok(DependencyStatus {
            ytdlp: true,
            aria2c: true,
            ffmpeg: true,
        })
    }

    /// URLのコンテンツタイプを検出する
    pub fn detect_content_type(&self, url: &str) -> Result<String> {
        // 実際の実装では、URLからコンテンツタイプを検出
        // ここではモック実装
        if url.contains("youtube.com") || url.contains("youtu.be") {
            Ok("YouTube".to_string())
        } else if url.ends_with(".mp4") {
            Ok("MP4".to_string())
        } else if url.ends_with(".mp3") {
            Ok("MP3".to_string())
        } else {
            Ok("Unknown".to_string())
        }
    }
}

// flutter_rust_bridge用の公開API関数

/// 新しいダウンロードを開始する
pub fn start_download(url: String, options: DownloadOptions) -> Result<String> {
    let download_manager = DOWNLOAD_MANAGER.lock().unwrap();
    download_manager.start_download(&url, &options)
}

/// ダウンロードの進捗を取得する
pub fn get_download_progress(download_id: String) -> Result<f32> {
    let download_manager = DOWNLOAD_MANAGER.lock().unwrap();
    download_manager.get_download_progress(&download_id)
}

/// ダウンロードをキャンセルする
pub fn cancel_download(download_id: String) -> Result<()> {
    let download_manager = DOWNLOAD_MANAGER.lock().unwrap();
    download_manager.cancel_download(&download_id)
}

/// ダウンロードを一時停止する
pub fn pause_download(download_id: String) -> Result<()> {
    let download_manager = DOWNLOAD_MANAGER.lock().unwrap();
    download_manager.pause_download(&download_id)
}

/// ダウンロードを再開する
pub fn resume_download(download_id: String) -> Result<()> {
    let download_manager = DOWNLOAD_MANAGER.lock().unwrap();
    download_manager.resume_download(&download_id)
}

/// すべてのダウンロードタスクを取得する
pub fn get_all_downloads() -> Vec<DownloadTask> {
    let download_manager = DOWNLOAD_MANAGER.lock().unwrap();
    download_manager.get_all_downloads()
}

/// 依存関係をチェックする
pub fn check_dependencies() -> Result<DependencyStatus> {
    let download_manager = DOWNLOAD_MANAGER.lock().unwrap();
    download_manager.check_dependencies()
}

/// URLのコンテンツタイプを検出する
pub fn detect_content_type(url: String) -> Result<String> {
    let download_manager = DOWNLOAD_MANAGER.lock().unwrap();
    download_manager.detect_content_type(&url)
}

/// ダウンロードマネージャーを初期化する
pub fn initialize() -> Result<()> {
    // 実際の実装では、ロギングの設定などを行う
    // ここでは特に何もしない
    Ok(())
}

/// バージョン情報を取得する
pub fn get_version() -> String {
    "0.1.0".to_string()
}