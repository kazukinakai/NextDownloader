use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use thiserror::Error;
use std::io;
use reqwest;

/// ダウンロードするコンテンツのタイプ
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContentType {
    /// 通常のMP4ファイル
    Mp4,
    /// HLSストリーミング (m3u8)
    Hls,
    /// MPEG-DASHストリーミング (mpd)
    Dash,
    /// YouTube動画
    YouTube,
    /// 不明なタイプ
    Unknown,
}

/// 動画フォーマット
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum VideoFormat {
    /// MP4フォーマット
    Mp4,
    /// MKVフォーマット
    Mkv,
    /// MP3フォーマット (音声のみ)
    Mp3,
}

/// ダウンロードオプション
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadOptions {
    /// 並列コネクション数
    pub connections: u32,
    /// ファイル分割数
    pub splits: u32,
    /// チャンクサイズ (MB)
    pub chunk_size: u32,
    /// リトライ待機時間 (秒)
    pub retry_wait: u32,
    /// 最大リトライ回数
    pub max_retries: u32,
    /// HTTP/2を使用
    pub use_http2: bool,
    /// QUIC (HTTP/3)を使用
    pub use_quic: bool,
    /// Keep-Aliveを使用
    pub use_keep_alive: bool,
    /// 出力フォーマット
    pub format: VideoFormat,
    /// Rustの直接ダウンロード機能を使用するか
    /// true: reqwestを使用した直接ダウンロード
    /// false: yt-dlpとaria2cを使用したダウンロード
    pub use_direct_download: Option<bool>,
    /// セグメントダウンロードのタイムアウト設定（秒）
    pub segment_timeout: Option<u64>,
    /// セグメントダウンロードのリトライ回数
    pub segment_retries: Option<u32>,
    /// セグメントダウンロードのリトライ間隔（ミリ秒）
    pub segment_retry_delay: Option<u64>,
}

impl Default for DownloadOptions {
    fn default() -> Self {
        Self {
            connections: 16,
            splits: 16,
            chunk_size: 4,
            retry_wait: 2,
            max_retries: 5,
            use_http2: true,
            use_quic: false,
            use_keep_alive: true,
            format: VideoFormat::Mp4,
            use_direct_download: Some(false),
            segment_timeout: Some(30),
            segment_retries: Some(3),
            segment_retry_delay: Some(1000),
        }
    }
}

/// ダウンロード関連のエラー
#[derive(Debug, Error, Clone)]
pub enum DownloadError {
    /// ファイルが見つからない
    #[error("ファイルが見つかりません")]
    FileNotFound,
    
    /// コンテンツタイプが不明
    #[error("コンテンツタイプが不明です")]
    UnknownContentType,
    
    /// プロセス実行失敗
    #[error("プロセスの実行に失敗: {0}")]
    ProcessFailed(String),
    
    /// I/Oエラー（文字列メッセージ）
    #[error("I/Oエラー: {0}")]
    IoError(String),
    
    /// ネットワークエラー
    #[error("ネットワークエラー: {0}")]
    NetworkError(String),
    
    /// シリアライゼーションエラー
    #[error("シリアライゼーションエラー: {0}")]
    SerializationError(String),
    
    /// ツールが見つからない
    #[error("必要なツールが見つかりません: {0}")]
    ToolNotFound(String),
    
    /// 範囲指定が無効
    #[error("無効な範囲指定: {0}")]
    InvalidRange(String),
    
    /// I/Oエラー
    #[error("I/Oエラー: {0}")]
    Io(#[from] Box<std::io::Error>),
    
    /// JSON解析エラー
    #[error("JSON解析エラー: {0}")]
    Json(#[from] serde_json::Error),
    
    /// 内部エラー
    #[error("内部エラー: {0}")]
    Internal(String),
    
    /// ダウンロード状態のエラー
    #[error("ダウンロード状態のエラー: {0}")]
    StateError(String),
}

/// 動画情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoInfo {
    /// タイトル
    pub title: Option<String>,
    /// 利用可能なフォーマット
    pub formats: Option<Vec<FormatInfo>>,
    /// 説明
    pub description: Option<String>,
    /// 長さ（秒）
    pub duration: Option<f64>,
    /// URL
    pub url: Option<String>,
}

/// フォーマット情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatInfo {
    /// フォーマットID
    pub format_id: Option<String>,
    /// URL
    pub url: Option<String>,
    /// マニフェストURL
    pub manifest_url: Option<String>,
    /// 幅
    pub width: Option<u32>,
    /// 高さ
    pub height: Option<u32>,
    /// 拡張子
    pub ext: Option<String>,
}

/// 進捗情報
#[derive(Debug, Clone, Serialize)]
pub struct ProgressInfo {
    /// 進捗（0.0〜1.0）
    pub progress: f64,
    /// ダウンロード速度
    pub speed: String,
    /// 残り時間
    pub eta: String,
}

/// プログレスコールバック型定義
pub type ProgressCallback = Box<dyn Fn(ProgressInfo) + Send + Sync>;

/// システムの状態
#[derive(Debug, Clone)]
pub enum SystemStatus {
    /// 準備完了
    Ready,
    /// 依存関係が足りない
    MissingDependencies {
        /// yt-dlpが使用可能か
        ytdlp: bool,
        /// aria2cが使用可能か
        aria2c: bool,
        /// ffmpegが使用可能か
        ffmpeg: bool,
    },
    /// 不明な状態
    Unknown,
}

impl SystemStatus {
    /// 準備完了かどうか
    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Ready)
    }
    
    /// 状態の説明
    pub fn description(&self) -> String {
        match self {
            Self::Ready => "システム準備完了".to_string(),
            Self::MissingDependencies { ytdlp, aria2c, ffmpeg } => {
                let mut missing = Vec::new();
                if !ytdlp { missing.push("yt-dlp") }
                if !aria2c { missing.push("aria2c") }
                if !ffmpeg { missing.push("ffmpeg") }
                format!("依存関係が不足しています: {}", missing.join(", "))
            }
            Self::Unknown => "システム状態不明".to_string(),
        }
    }
}
