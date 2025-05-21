//! # DASHダウンローダーモジュール
//! 
//! Dynamic Adaptive Streaming over HTTP (DASH) のダウンロードを実装します。

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Write};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use bytes::Bytes;
use dash_mpd::{MPD, fetch::DashDownloader as DashMpdDownloader, fetch::DashDownloaderOptions};
use futures::{stream, StreamExt};
use log::{debug, error, info, warn};
use reqwest::{Client, StatusCode};
use tokio::sync::mpsc;
use tokio::time;
use tokio::io::AsyncWriteExt;
use tokio::task::JoinHandle;
use url::Url;

use crate::error::DownloaderError;
use crate::downloader::{Downloader, DownloadOptions};
use crate::{DownloadInfo, DownloadProgress, DownloadStatus};
use super::common::{StreamingOptions, StreamSegment, StreamingError, StreamType, QualityLevel, MuxerType};

/// DASHダウンローダー
pub struct DashDownloader {
    /// HTTP クライアント
    client: Client,
    /// ダウンロード情報
    downloads: Arc<Mutex<HashMap<String, DownloadInfo>>>,
    /// アクティブなダウンロードタスク
    tasks: Arc<Mutex<HashMap<String, Vec<JoinHandle<Result<(), StreamingError>>>>>>,
}

impl DashDownloader {
    /// 新しいDASHダウンローダーを作成します
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
            downloads: Arc::new(Mutex::new(HashMap::new())),
            tasks: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// DASHマニフェストを解析します
    pub async fn parse_manifest(&self, url: &str) -> Result<DashManifest, StreamingError> {
        // DASHダウンローダーオプションの作成
        let mpd_options = DashDownloaderOptions::default();
        
        // dash-mpdライブラリを使用してマニフェストを解析
        let downloader = DashMpdDownloader::new(&mpd_options);
        
        let mpd = downloader.get_mpd(url)
            .await
            .map_err(|e| StreamingError::ManifestParseError(format!("MPDの取得に失敗しました: {}", e)))?;
        
        // アダプテーションセットの解析
        let mut video_adaptations = Vec::new();
        let mut audio_adaptations = Vec::new();
        let mut subtitle_adaptations = Vec::new();
        
        // MPD情報の抽出
        for period in mpd.periods() {
            for adaptation_set in period.adaptation_sets() {
                let content_type = adaptation_set.content_type();
                let mime_type = adaptation_set.mime_type();
                let lang = adaptation_set.lang();
                
                // アダプテーションセットのタイプを判定
                let stream_type = match content_type.unwrap_or_default().as_str() {
                    "video" => StreamType::Video,
                    "audio" => StreamType::Audio,
                    "text" | "subtitle" => StreamType::Subtitle,
                    _ => {
                        // MIMEタイプからの判定
                        match mime_type.unwrap_or_default().as_str() {
                            "video/mp4" => StreamType::Video,
                            "audio/mp4" => StreamType::Audio,
                            "text/vtt" | "application/ttml+xml" => StreamType::Subtitle,
                            _ => StreamType::Other,
                        }
                    }
                };
                
                // 各アダプテーションセットの表現（レプレゼンテーション）を解析
                let mut representations = Vec::new();
                for rep in adaptation_set.representations() {
                    let id = rep.id().unwrap_or_default().to_string();
                    let bandwidth = rep.bandwidth() as u64;
                    let codec = rep.codecs().map(|c| c.to_string());
                    
                    // 解像度の取得
                    let resolution = match (rep.width(), rep.height()) {
                        (Some(w), Some(h)) => Some((w as u32, h as u32)),
                        _ => None,
                    };
                    
                    // 表現を追加
                    representations.push(DashRepresentation {
                        id,
                        bandwidth,
                        codec,
                        resolution,
                        mime_type: rep.mime_type().map(|m| m.to_string()),
                    });
                }
                
                // アダプテーションセットを適切なリストに追加
                let adaptation = DashAdaptationSet {
                    id: adaptation_set.id().unwrap_or_default().to_string(),
                    stream_type,
                    language: lang.map(|l| l.to_string()),
                    representations,
                };
                
                match stream_type {
                    StreamType::Video => video_adaptations.push(adaptation),
                    StreamType::Audio => audio_adaptations.push(adaptation),
                    StreamType::Subtitle => subtitle_adaptations.push(adaptation),
                    _ => {}
                }
            }
        }
        
        // DASHマニフェストを作成
        let manifest = DashManifest {
            url: url.to_string(),
            mpd,
            duration: mpd.media_presentation_duration().map(|d| d.as_secs() as f64),
            is_live: mpd.profiles().any(|p| p.contains("dynamic")),
            video_adaptations,
            audio_adaptations,
            subtitle_adaptations,
        };
        
        Ok(manifest)
    }
    
    /// DASHストリームをダウンロードします
    pub async fn download_stream(
        &self,
        manifest_url: &str,
        options: StreamingOptions,
    ) -> Result<(), StreamingError> {
        // マニフェストを解析
        let manifest = self.parse_manifest(manifest_url).await?;
        
        // 一時ディレクトリの作成
        fs::create_dir_all(&options.temp_dir)
            .map_err(|e| StreamingError::FileError(format!("一時ディレクトリの作成に失敗しました: {}", e)))?;
        
        // 最適な品質を選択
        let (selected_video, selected_audio) = self.select_best_quality(&manifest, &options)?;
        
        // ダウンロードオプションの作成
        let dash_options = DashDownloaderOptions::default();
        let downloader = DashMpdDownloader::new(&dash_options);
        
        // ビデオと音声のファイルパス
        let video_path = options.temp_dir.join("video.mp4");
        let audio_path = options.temp_dir.join("audio.mp4");
        
        // 進行状況のトラッキング
        let progress_callback = options.progress_callback.clone();
        let total_size = selected_video.as_ref().map(|v| v.bandwidth).unwrap_or(0) +
                        selected_audio.as_ref().map(|a| a.bandwidth).unwrap_or(0);
        
        // ビデオのダウンロード（選択されている場合）
        if let Some(video) = selected_video.as_ref() {
            info!("ビデオ表現のダウンロード開始: id={}, bandwidth={}bps", video.id, video.bandwidth);
            
            // dash-mpdのダウンローダーを使用してビデオをダウンロード
            downloader.download_representation(
                &manifest.mpd,
                &video.id,
                video_path.to_str().unwrap(),
            )
            .await
            .map_err(|e| StreamingError::SegmentDownloadError(format!("ビデオのダウンロードに失敗しました: {}", e)))?;
            
            info!("ビデオ表現のダウンロード完了: {}", video_path.display());
        }
        
        // 音声のダウンロード（選択されている場合かつビデオのみではない場合）
        if let Some(audio) = selected_audio.as_ref() {
            if !options.video_only {
                info!("音声表現のダウンロード開始: id={}, bandwidth={}bps", audio.id, audio.bandwidth);
                
                // dash-mpdのダウンローダーを使用して音声をダウンロード
                downloader.download_representation(
                    &manifest.mpd,
                    &audio.id,
                    audio_path.to_str().unwrap(),
                )
                .await
                .map_err(|e| StreamingError::SegmentDownloadError(format!("音声のダウンロードに失敗しました: {}", e)))?;
                
                info!("音声表現のダウンロード完了: {}", audio_path.display());
            }
        }
        
        // 出力ディレクトリの作成
        let output_dir = options.output_file.parent()
            .unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(output_dir)
            .map_err(|e| StreamingError::FileError(format!("出力ディレクトリの作成に失敗しました: {}", e)))?;
        
        // ビデオと音声のマージ（必要な場合）
        if selected_video.is_some() && selected_audio.is_some() && !options.video_only && !options.audio_only {
            // FFmpegを使用してビデオと音声をマージ
            if self.is_ffmpeg_available() {
                info!("ビデオと音声のマージを開始");
                self.merge_with_ffmpeg(&video_path, &audio_path, &options.output_file).await?;
                info!("ビデオと音声のマージが完了しました: {}", options.output_file.display());
            } else {
                // FFmpegが利用できない場合はエラー
                return Err(StreamingError::OtherError(
                    "ビデオと音声のマージにはFFmpegが必要です".to_string()
                ));
            }
        } else if selected_video.is_some() {
            // ビデオのみの場合は単純にコピー
            fs::copy(&video_path, &options.output_file)
                .map_err(|e| StreamingError::FileError(format!("ファイルのコピーに失敗しました: {}", e)))?;
        } else if selected_audio.is_some() {
            // 音声のみの場合は単純にコピー
            fs::copy(&audio_path, &options.output_file)
                .map_err(|e| StreamingError::FileError(format!("ファイルのコピーに失敗しました: {}", e)))?;
        }
        
        // 一時ファイルのクリーンアップ
        if options.cleanup_temp_files {
            if selected_video.is_some() {
                let _ = fs::remove_file(&video_path);
            }
            
            if selected_audio.is_some() {
                let _ = fs::remove_file(&audio_path);
            }
            
            let _ = fs::remove_dir(&options.temp_dir);
        }
        
        Ok(())
    }
    
    /// 最適な品質を選択します
    fn select_best_quality(
        &self,
        manifest: &DashManifest,
        options: &StreamingOptions,
    ) -> Result<(Option<DashRepresentation>, Option<DashRepresentation>), StreamingError> {
        // ビデオの選択
        let selected_video = if !options.audio_only && !manifest.video_adaptations.is_empty() {
            let adaptation = &manifest.video_adaptations[0];
            
            // 利用可能な表現をフィルタリング
            let mut representations = adaptation.representations.clone();
            
            // 最大解像度の制限がある場合、それを超える表現を除外
            if let Some(max_res) = options.max_resolution {
                representations.retain(|r| {
                    if let Some((width, height)) = r.resolution {
                        width <= max_res.0 && height <= max_res.1
                    } else {
                        true
                    }
                });
            }
            
            // 最小解像度の制限がある場合、それ未満の表現を除外
            if let Some(min_res) = options.min_resolution {
                representations.retain(|r| {
                    if let Some((width, height)) = r.resolution {
                        width >= min_res.0 && height >= min_res.1
                    } else {
                        true
                    }
                });
            }
            
            // 最大ビットレートの制限がある場合、それを超える表現を除外
            if let Some(max_bitrate) = options.max_bitrate {
                representations.retain(|r| r.bandwidth <= max_bitrate * 1000);
            }
            
            // 最小ビットレートの制限がある場合、それ未満の表現を除外
            if let Some(min_bitrate) = options.min_bitrate {
                representations.retain(|r| r.bandwidth >= min_bitrate * 1000);
            }
            
            // 残った表現がない場合は元のリストを使用
            if representations.is_empty() {
                representations = adaptation.representations.clone();
            }
            
            // フィルタリング後の表現から最適なものを選択
            // 解像度優先（解像度が最も高いもの）
            representations.sort_by(|a, b| {
                let a_res = a.resolution.map(|(w, h)| w * h).unwrap_or(0);
                let b_res = b.resolution.map(|(w, h)| w * h).unwrap_or(0);
                b_res.cmp(&a_res)
            });
            
            representations.first().cloned()
        } else {
            None
        };
        
        // 音声の選択
        let selected_audio = if !options.video_only && !manifest.audio_adaptations.is_empty() {
            let adaptation = &manifest.audio_adaptations[0];
            
            // 言語の優先（指定があれば）
            // ここでは単純化のために最初の音声アダプテーションセットを使用
            
            // 利用可能な表現をフィルタリング
            let mut representations = adaptation.representations.clone();
            
            // 最大ビットレートの制限がある場合、それを超える表現を除外
            if let Some(max_bitrate) = options.max_bitrate {
                representations.retain(|r| r.bandwidth <= max_bitrate * 1000);
            }
            
            // 最小ビットレートの制限がある場合、それ未満の表現を除外
            if let Some(min_bitrate) = options.min_bitrate {
                representations.retain(|r| r.bandwidth >= min_bitrate * 1000);
            }
            
            // 残った表現がない場合は元のリストを使用
            if representations.is_empty() {
                representations = adaptation.representations.clone();
            }
            
            // フィルタリング後の表現から最適なものを選択
            // ビットレート優先（ビットレートが最も高いもの）
            representations.sort_by(|a, b| {
                b.bandwidth.cmp(&a.bandwidth)
            });
            
            representations.first().cloned()
        } else {
            None
        };
        
        if selected_video.is_none() && selected_audio.is_none() {
            return Err(StreamingError::ManifestParseError("適切な表現が見つかりませんでした".to_string()));
        }
        
        Ok((selected_video, selected_audio))
    }
    
    /// FFmpegが利用可能かどうかを確認します
    fn is_ffmpeg_available(&self) -> bool {
        // FFmpegが利用可能かどうかをチェック
        // 実際の実装ではパスをチェックするか、システムコールを使って確認する
        cfg!(feature = "ffmpeg")
    }
    
    /// FFmpegを使用してビデオと音声をマージします
    async fn merge_with_ffmpeg(
        &self,
        video_path: &Path,
        audio_path: &Path,
        output_file: &Path,
    ) -> Result<(), StreamingError> {
        #[cfg(feature = "ffmpeg")]
        {
            // FFmpegコマンドの構築と実行（非同期プロセス実行）
            let ffmpeg_args = vec![
                "-i", video_path.to_str().unwrap(),
                "-i", audio_path.to_str().unwrap(),
                "-c", "copy",
                "-movflags", "faststart",
                "-y", // 既存ファイルを上書き
                output_file.to_str().unwrap(),
            ];
            
            // Tokioを使ってFFmpegを非同期で実行
            let ffmpeg_result = tokio::process::Command::new("ffmpeg")
                .args(&ffmpeg_args)
                .output()
                .await
                .map_err(|e| StreamingError::OtherError(format!("FFmpegの実行に失敗しました: {}", e)))?;
            
            if !ffmpeg_result.status.success() {
                let stderr = String::from_utf8_lossy(&ffmpeg_result.stderr);
                return Err(StreamingError::OtherError(format!("FFmpegの実行に失敗しました: {}", stderr)));
            }
            
            Ok(())
        }
        
        #[cfg(not(feature = "ffmpeg"))]
        {
            Err(StreamingError::OtherError("FFmpeg機能が有効になっていません".to_string()))
        }
    }
}

/// DASHマニフェスト
#[derive(Debug, Clone)]
pub struct DashManifest {
    /// マニフェストのURL
    pub url: String,
    /// MPDオブジェクト
    pub mpd: MPD,
    /// メディアの合計時間（秒）
    pub duration: Option<f64>,
    /// ライブストリームかどうか
    pub is_live: bool,
    /// ビデオのアダプテーションセット
    pub video_adaptations: Vec<DashAdaptationSet>,
    /// 音声のアダプテーションセット
    pub audio_adaptations: Vec<DashAdaptationSet>,
    /// 字幕のアダプテーションセット
    pub subtitle_adaptations: Vec<DashAdaptationSet>,
}

/// DASHアダプテーションセット
#[derive(Debug, Clone)]
pub struct DashAdaptationSet {
    /// アダプテーションセットのID
    pub id: String,
    /// ストリームのタイプ
    pub stream_type: StreamType,
    /// 言語
    pub language: Option<String>,
    /// 表現（異なる品質レベル）
    pub representations: Vec<DashRepresentation>,
}

/// DASH表現
#[derive(Debug, Clone)]
pub struct DashRepresentation {
    /// 表現のID
    pub id: String,
    /// ビットレート（bps）
    pub bandwidth: u64,
    /// コーデック情報
    pub codec: Option<String>,
    /// 解像度（幅x高さ）
    pub resolution: Option<(u32, u32)>,
    /// MIMEタイプ
    pub mime_type: Option<String>,
}

impl Default for DashDownloader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    // テストはここに追加
}