//! # ストリーミング共通モジュール
//! 
//! HLSとDASHでの共通部分を定義します。

use std::path::PathBuf;
use std::time::Duration;
use serde::{Deserialize, Serialize};

/// ストリームセグメント情報
#[derive(Debug, Clone)]
pub struct StreamSegment {
    /// セグメントのURL
    pub url: String,
    /// セグメントの期間（秒）
    pub duration: f64,
    /// セグメントのシーケンス番号
    pub sequence: u64,
    /// セグメントのバイトレンジ（ある場合）
    pub byte_range: Option<(u64, u64)>,
    /// セグメントのタイトル（HLSの場合）
    pub title: Option<String>,
    /// セグメントの解像度（ビデオの場合）
    pub resolution: Option<(u32, u32)>,
    /// セグメントのビットレート（Kbps）
    pub bandwidth: Option<u64>,
    /// セグメントのコーデック情報
    pub codec: Option<String>,
}

/// ストリーミングダウンロードのオプション
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingOptions {
    /// セグメントの保存先ディレクトリ
    pub temp_dir: PathBuf,
    /// 出力ファイルパス
    pub output_file: PathBuf,
    /// 最大同時ダウンロード数
    pub max_concurrent_downloads: usize,
    /// セグメントのダウンロード再試行回数
    pub retry_count: u32,
    /// セグメントのダウンロードタイムアウト
    pub segment_timeout: Duration,
    /// マニフェストの再読み込み間隔（ライブストリームの場合）
    pub manifest_reload_interval: Option<Duration>,
    /// 動画のみをダウンロードする（音声なし）
    pub video_only: bool,
    /// 音声のみをダウンロードする（動画なし）
    pub audio_only: bool,
    /// 最大解像度
    pub max_resolution: Option<(u32, u32)>,
    /// 最小解像度
    pub min_resolution: Option<(u32, u32)>,
    /// ダウンロード速度制限（bytes/sec）
    pub bandwidth_limit: Option<u64>,
    /// 最大ビットレート（Kbps）
    pub max_bitrate: Option<u64>,
    /// 最小ビットレート（Kbps）
    pub min_bitrate: Option<u64>,
    /// プログレスコールバック
    pub progress_callback: Option<Box<dyn Fn(f64, u64, Option<u64>) + Send + Sync>>,
    /// ダウンロード後に一時ファイルを削除するかどうか
    pub cleanup_temp_files: bool,
}

impl Default for StreamingOptions {
    fn default() -> Self {
        let temp_dir = std::env::temp_dir().join("nextdownloader");
        Self {
            temp_dir,
            output_file: PathBuf::from("output.mp4"),
            max_concurrent_downloads: 4,
            retry_count: 3,
            segment_timeout: Duration::from_secs(30),
            manifest_reload_interval: None,
            video_only: false,
            audio_only: false,
            max_resolution: None,
            min_resolution: None,
            bandwidth_limit: None,
            max_bitrate: None,
            min_bitrate: None,
            progress_callback: None,
            cleanup_temp_files: true,
        }
    }
}

/// ストリームの品質レベル
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityLevel {
    /// 品質レベルのID
    pub id: String,
    /// 解像度（幅x高さ）
    pub resolution: Option<(u32, u32)>,
    /// ビットレート（Kbps）
    pub bitrate: u64,
    /// コーデック情報
    pub codec: Option<String>,
    /// 言語（音声・字幕の場合）
    pub language: Option<String>,
    /// ストリームのタイプ（ビデオ、オーディオなど）
    pub stream_type: StreamType,
}

/// ストリームのタイプ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StreamType {
    /// ビデオストリーム
    Video,
    /// オーディオストリーム
    Audio,
    /// 字幕ストリーム
    Subtitle,
    /// 複合ストリーム（ビデオ+オーディオ）
    Muxed,
    /// その他のストリーム
    Other,
}

/// DRM情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrmInfo {
    /// DRMのタイプ
    pub drm_type: DrmType,
    /// ライセンスサーバーのURL
    pub license_url: Option<String>,
    /// その他のDRM固有データ
    pub data: Option<serde_json::Value>,
}

/// DRMのタイプ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DrmType {
    /// Widevine
    Widevine,
    /// PlayReady
    PlayReady,
    /// FairPlay
    FairPlay,
    /// その他のDRM
    Other,
    /// DRMなし
    None,
}

impl Default for DrmType {
    fn default() -> Self {
        DrmType::None
    }
}

/// エラータイプ
#[derive(Debug, thiserror::Error)]
pub enum StreamingError {
    /// マニフェストの解析エラー
    #[error("マニフェスト解析エラー: {0}")]
    ManifestParseError(String),
    
    /// セグメントのダウンロードエラー
    #[error("セグメントダウンロードエラー: {0}")]
    SegmentDownloadError(String),
    
    /// ファイル操作エラー
    #[error("ファイル操作エラー: {0}")]
    FileError(String),
    
    /// HTTP通信エラー
    #[error("HTTP通信エラー: {0}")]
    HttpError(String),
    
    /// DRMエラー
    #[error("DRMエラー: {0}")]
    DrmError(String),
    
    /// その他のエラー
    #[error("その他のエラー: {0}")]
    OtherError(String),
}

/// Stream Muxer
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MuxerType {
    /// FFmpeg
    FFmpeg,
    /// MP4Box
    MP4Box,
    /// その他
    Other,
}

impl Default for MuxerType {
    fn default() -> Self {
        MuxerType::FFmpeg
    }
}