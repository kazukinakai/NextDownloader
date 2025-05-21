//! # NextDownloader Core
//! 
//! NextDownloaderのコアライブラリです。
//! このライブラリは、様々な形式のコンテンツを簡単かつ高速にダウンロードするための
//! 機能を提供します。

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use anyhow::Result;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use uuid::Uuid;

// モジュールのエクスポート
pub mod downloader;
pub mod content_type;
pub mod error;
pub mod utils;
pub mod config;
pub mod streaming;
pub mod encoding;

// 再エクスポート
pub use content_type::ContentType;
pub use downloader::{Downloader, DownloadManager, DownloadOptions, VideoFormat};
pub use error::ErrorCode;
pub use streaming::{hls::HlsDownloader, dash::DashDownloader, common::StreamingOptions};
pub use encoding::{EncodingManager, EncodingOptions, VideoInfo};

/// ダウンロードの進捗状況
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    /// ダウンロードID
    pub id: String,
    /// 進捗率（0.0 - 1.0）
    pub progress: f64,
    /// ダウンロード速度（bytes/sec）
    pub speed: Option<u64>,
    /// 推定残り時間（秒）
    pub eta: Option<u64>,
    /// ダウンロード済みサイズ（bytes）
    pub downloaded_size: u64,
    /// 合計サイズ（bytes）
    pub total_size: Option<u64>,
    /// ステータスメッセージ
    pub status_message: Option<String>,
}

/// ダウンロードの状態
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DownloadStatus {
    /// 初期化中
    Initializing,
    /// ダウンロード中
    Downloading,
    /// 一時停止中
    Paused,
    /// 完了
    Completed,
    /// エラー
    Error,
    /// キャンセル
    Cancelled,
}

/// ダウンロード情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadInfo {
    /// ダウンロードID
    pub id: String,
    /// URL
    pub url: String,
    /// 保存先パス
    pub destination: PathBuf,
    /// コンテンツタイプ
    pub content_type: ContentType,
    /// ステータス
    pub status: DownloadStatus,
    /// 進捗情報
    pub progress: DownloadProgress,
    /// 作成日時（ISO 8601形式）
    pub created_at: String,
    /// 完了日時（ISO 8601形式）
    pub completed_at: Option<String>,
    /// エラーメッセージ
    pub error_message: Option<String>,
}

/// ダウンロードマネージャーの設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadManagerConfig {
    /// 最大同時ダウンロード数
    pub max_concurrent_downloads: usize,
    /// 一時ファイルディレクトリ
    pub temp_dir: Option<PathBuf>,
    /// 自動再開を有効にする
    pub auto_resume: bool,
    /// ダウンロード完了時に通知する
    pub notify_on_completion: bool,
    /// アーカイブを自動的に解凍する
    pub auto_extract_archives: bool,
}

impl Default for DownloadManagerConfig {
    fn default() -> Self {
        Self {
            max_concurrent_downloads: 3,
            temp_dir: None,
            auto_resume: true,
            notify_on_completion: true,
            auto_extract_archives: false,
        }
    }
}

/// 依存関係のステータス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyStatus {
    /// yt-dlpがインストールされているか
    pub ytdlp: bool,
    /// aria2cがインストールされているか
    pub aria2c: bool,
    /// ffmpegがインストールされているか
    pub ffmpeg: bool,
}

/// NextDownloaderのバージョン情報を返します
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!version().is_empty());
    }
}