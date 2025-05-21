//! # HLSダウンローダーモジュール
//! 
//! HTTP Live Streaming (HLS) のダウンロードを実装します。

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Write};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use bytes::Bytes;
use futures::{stream, StreamExt};
use log::{debug, error, info, warn};
use m3u8_rs::{MediaPlaylist, MasterPlaylist, Playlist, MediaSegment};
use reqwest::{Client, StatusCode};
use tokio::sync::mpsc;
use tokio::time;
use tokio::io::AsyncWriteExt;
use tokio::task::JoinHandle;
use url::Url;

use crate::error::DownloaderError;
use crate::downloader::{Downloader, DownloadOptions};
use crate::{DownloadInfo, DownloadProgress, DownloadStatus};
use super::common::{StreamingOptions, StreamSegment, StreamingError, MuxerType};

/// HLSダウンローダー
pub struct HlsDownloader {
    /// HTTP クライアント
    client: Client,
    /// ダウンロード情報
    downloads: Arc<Mutex<HashMap<String, DownloadInfo>>>,
    /// アクティブなダウンロードタスク
    tasks: Arc<Mutex<HashMap<String, Vec<JoinHandle<Result<(), StreamingError>>>>>>,
}

impl HlsDownloader {
    /// 新しいHLSダウンローダーを作成します
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
    
    /// HLSマニフェストを解析します
    pub async fn parse_manifest(&self, url: &str) -> Result<HlsManifest, StreamingError> {
        // マニフェストをダウンロード
        let response = self.client.get(url)
            .send()
            .await
            .map_err(|e| StreamingError::HttpError(e.to_string()))?;
        
        if !response.status().is_success() {
            return Err(StreamingError::HttpError(format!(
                "マニフェストの取得に失敗しました: ステータスコード {}", 
                response.status()
            )));
        }
        
        let manifest_content = response.text()
            .await
            .map_err(|e| StreamingError::HttpError(e.to_string()))?;
        
        // HLSマニフェストを解析
        match m3u8_rs::parse_playlist(manifest_content.as_bytes()) {
            Ok((_, Playlist::MasterPlaylist(master))) => {
                // マスタープレイリストの場合
                let mut variants = Vec::new();
                
                for variant in &master.variants {
                    let url = if variant.uri.starts_with("http") {
                        variant.uri.clone()
                    } else {
                        // 相対URLの場合は基準URLと結合
                        let base_url = Url::parse(url)
                            .map_err(|e| StreamingError::ManifestParseError(e.to_string()))?;
                        base_url.join(&variant.uri)
                            .map_err(|e| StreamingError::ManifestParseError(e.to_string()))?
                            .to_string()
                    };
                    
                    let resolution = variant.resolution.as_ref().map(|r| (r.width, r.height));
                    let bandwidth = variant.bandwidth;
                    let codec = variant.codecs.clone();
                    
                    variants.push(HlsVariant {
                        url,
                        resolution,
                        bandwidth,
                        codec,
                    });
                }
                
                Ok(HlsManifest::Master(HlsMaster {
                    url: url.to_string(),
                    variants,
                }))
            },
            Ok((_, Playlist::MediaPlaylist(media))) => {
                // メディアプレイリストの場合
                let mut segments = Vec::new();
                let base_url = Url::parse(url)
                    .map_err(|e| StreamingError::ManifestParseError(e.to_string()))?;
                
                for (i, segment) in media.segments.iter().enumerate() {
                    let segment_url = if segment.uri.starts_with("http") {
                        segment.uri.clone()
                    } else {
                        // 相対URLの場合は基準URLと結合
                        base_url.join(&segment.uri)
                            .map_err(|e| StreamingError::ManifestParseError(e.to_string()))?
                            .to_string()
                    };
                    
                    let byte_range = segment.byte_range.as_ref().map(|range| {
                        (range.start, range.start + range.length)
                    });
                    
                    segments.push(StreamSegment {
                        url: segment_url,
                        duration: segment.duration,
                        sequence: i as u64,
                        byte_range,
                        title: segment.title.clone(),
                        resolution: None,
                        bandwidth: None,
                        codec: None,
                    });
                }
                
                Ok(HlsManifest::Media(HlsMedia {
                    url: url.to_string(),
                    segments,
                    target_duration: media.target_duration,
                    version: media.version,
                    sequence_no: media.media_sequence,
                    is_live: media.end_list.is_none(),
                }))
            },
            Err(e) => {
                Err(StreamingError::ManifestParseError(format!("マニフェストの解析に失敗しました: {}", e)))
            }
        }
    }
    
    /// HLSストリームをダウンロードします
    pub async fn download_stream(
        &self,
        manifest_url: &str,
        options: StreamingOptions,
    ) -> Result<(), StreamingError> {
        // マニフェストを解析
        let manifest = self.parse_manifest(manifest_url).await?;
        
        match manifest {
            HlsManifest::Master(master) => {
                // マスタープレイリストの場合、適切な品質を選択
                if master.variants.is_empty() {
                    return Err(StreamingError::ManifestParseError("マスタープレイリストにバリアントがありません".to_string()));
                }
                
                // 解像度とビットレートに基づいて最適なバリアントを選択
                let selected_variant = self.select_best_variant(&master.variants, &options)?;
                info!("選択されたHLSバリアント: {:?}", selected_variant);
                
                // 選択されたバリアントをダウンロード
                self.download_stream(&selected_variant.url, options).await
            },
            HlsManifest::Media(media) => {
                // メディアプレイリストの場合、直接セグメントをダウンロード
                self.download_segments(&media, options).await
            }
        }
    }
    
    /// 最適なバリアントを選択します
    fn select_best_variant(
        &self,
        variants: &[HlsVariant],
        options: &StreamingOptions,
    ) -> Result<HlsVariant, StreamingError> {
        // ユーザーが設定した解像度制限に基づいてフィルタリング
        let mut filtered_variants = variants.to_vec();
        
        // 最大解像度の制限がある場合、それを超えるバリアントを除外
        if let Some(max_res) = options.max_resolution {
            filtered_variants.retain(|v| {
                if let Some((width, height)) = v.resolution {
                    width <= max_res.0 && height <= max_res.1
                } else {
                    true
                }
            });
        }
        
        // 最小解像度の制限がある場合、それ未満のバリアントを除外
        if let Some(min_res) = options.min_resolution {
            filtered_variants.retain(|v| {
                if let Some((width, height)) = v.resolution {
                    width >= min_res.0 && height >= min_res.1
                } else {
                    true
                }
            });
        }
        
        // 最大ビットレートの制限がある場合、それを超えるバリアントを除外
        if let Some(max_bitrate) = options.max_bitrate {
            filtered_variants.retain(|v| v.bandwidth <= max_bitrate * 1000);
        }
        
        // 最小ビットレートの制限がある場合、それ未満のバリアントを除外
        if let Some(min_bitrate) = options.min_bitrate {
            filtered_variants.retain(|v| v.bandwidth >= min_bitrate * 1000);
        }
        
        // 残ったバリアントがない場合は元のリストを使用
        if filtered_variants.is_empty() {
            filtered_variants = variants.to_vec();
        }
        
        // フィルタリング後のバリアントから最適なものを選択
        // 解像度優先（解像度が最も高いもの）
        filtered_variants.sort_by(|a, b| {
            let a_res = a.resolution.map(|(w, h)| w * h).unwrap_or(0);
            let b_res = b.resolution.map(|(w, h)| w * h).unwrap_or(0);
            b_res.cmp(&a_res)
        });
        
        Ok(filtered_variants.first().unwrap().clone())
    }
    
    /// HLSセグメントをダウンロードします
    async fn download_segments(
        &self,
        media: &HlsMedia,
        options: StreamingOptions,
    ) -> Result<(), StreamingError> {
        // 一時ディレクトリの作成
        fs::create_dir_all(&options.temp_dir)
            .map_err(|e| StreamingError::FileError(format!("一時ディレクトリの作成に失敗しました: {}", e)))?;
        
        let segments = &media.segments;
        let total_segments = segments.len();
        let total_duration: f64 = segments.iter().map(|s| s.duration).sum();
        
        info!("ダウンロード開始: {} セグメント, 総時間: {:.2}秒", total_segments, total_duration);
        
        // 同時ダウンロード数の制限
        let concurrent_limit = options.max_concurrent_downloads;
        
        // セグメントの並列ダウンロード
        let mut segment_files = Vec::with_capacity(total_segments);
        let mut downloaded_duration = 0.0;
        let start_time = Instant::now();
        
        let client = Client::new();
        
        // 進捗コールバック用の情報
        let progress_callback = options.progress_callback.clone();
        
        // セグメントを並列ダウンロード
        let results = stream::iter(segments.iter().enumerate())
            .map(|(i, segment)| {
                let client = client.clone();
                let segment_url = segment.url.clone();
                let byte_range = segment.byte_range;
                let temp_dir = options.temp_dir.clone();
                let retry_count = options.retry_count;
                let segment_timeout = options.segment_timeout;
                
                // 各セグメントのダウンロード関数
                async move {
                    let segment_file = temp_dir.join(format!("segment_{:04}.ts", i));
                    
                    // リトライロジック
                    let mut attempts = 0;
                    let mut last_error = None;
                    
                    while attempts < retry_count {
                        match download_segment(
                            &client,
                            &segment_url,
                            byte_range,
                            &segment_file,
                            segment_timeout,
                        ).await {
                            Ok(_) => {
                                return Ok((i, segment_file, segment.duration));
                            },
                            Err(e) => {
                                last_error = Some(e);
                                attempts += 1;
                                if attempts < retry_count {
                                    // 指数バックオフ
                                    let backoff = Duration::from_millis(500 * 2u64.pow(attempts as u32));
                                    tokio::time::sleep(backoff).await;
                                }
                            }
                        }
                    }
                    
                    Err(last_error.unwrap_or_else(|| {
                        StreamingError::SegmentDownloadError(
                            format!("セグメント {} のダウンロードに失敗しました", i)
                        )
                    }))
                }
            })
            .buffer_unordered(concurrent_limit)
            .collect::<Vec<_>>()
            .await;
        
        // エラーチェック
        let mut download_results = Vec::new();
        for result in results {
            match result {
                Ok(data) => download_results.push(data),
                Err(e) => {
                    // すべての一時ファイルをクリーンアップ
                    if options.cleanup_temp_files {
                        for (_, file, _) in &segment_files {
                            let _ = fs::remove_file(file);
                        }
                    }
                    return Err(e);
                }
            }
        }
        
        // ダウンロードしたセグメントをソート（順序が重要）
        download_results.sort_by_key(|(idx, _, _)| *idx);
        segment_files = download_results.into_iter()
            .map(|(_, file, duration)| (file, duration))
            .collect();
        
        // すべてのセグメントが正常にダウンロードされた
        info!("すべてのセグメントをダウンロードしました: {} ファイル", segment_files.len());
        
        // セグメントを結合
        let output_dir = options.output_file.parent()
            .unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(output_dir)
            .map_err(|e| StreamingError::FileError(format!("出力ディレクトリの作成に失敗しました: {}", e)))?;
        
        // 結合方法を選択
        // 1. 単純な連結（最も単純だが、一部の動画では問題が発生する可能性あり）
        // 2. FFmpegを使用した結合（推奨）
        
        if self.is_ffmpeg_available() {
            // FFmpegを使用した結合
            self.concat_with_ffmpeg(&segment_files, &options.output_file).await?;
        } else {
            // 単純な連結
            self.concat_segments(&segment_files, &options.output_file)?;
        }
        
        // 一時ファイルのクリーンアップ
        if options.cleanup_temp_files {
            for (file, _) in &segment_files {
                let _ = fs::remove_file(file);
            }
            let _ = fs::remove_dir(&options.temp_dir);
        }
        
        // 完了メッセージ
        let elapsed = start_time.elapsed();
        info!(
            "ダウンロード完了: {} ({:.2} MB) - 所要時間: {:.2}秒",
            options.output_file.display(),
            fs::metadata(&options.output_file)
                .map(|m| m.len() as f64 / 1_048_576.0)
                .unwrap_or(0.0),
            elapsed.as_secs_f64()
        );
        
        Ok(())
    }
    
    /// FFmpegが利用可能かどうかを確認します
    fn is_ffmpeg_available(&self) -> bool {
        // FFmpegが利用可能かどうかをチェック
        // 実際の実装ではパスをチェックするか、システムコールを使って確認する
        cfg!(feature = "ffmpeg")
    }
    
    /// FFmpegを使用してセグメントを結合します
    async fn concat_with_ffmpeg(
        &self,
        segment_files: &[(PathBuf, f64)],
        output_file: &Path,
    ) -> Result<(), StreamingError> {
        #[cfg(feature = "ffmpeg")]
        {
            // FFmpegの初期化
            use rust_ffmpeg::format::input;
            use rust_ffmpeg::format::output;
            
            // セグメントリストファイルの作成
            let temp_list = output_file.with_extension("list");
            let mut list_file = File::create(&temp_list)
                .map_err(|e| StreamingError::FileError(format!("リストファイルの作成に失敗しました: {}", e)))?;
            
            for (file, _) in segment_files {
                writeln!(list_file, "file '{}'", file.display())
                    .map_err(|e| StreamingError::FileError(format!("リストファイルの書き込みに失敗しました: {}", e)))?;
            }
            
            // FFmpegコマンドの構築と実行（非同期プロセス実行）
            let ffmpeg_args = vec![
                "-f", "concat",
                "-safe", "0",
                "-i", temp_list.to_str().unwrap(),
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
            
            // リストファイルの削除
            let _ = fs::remove_file(temp_list);
            
            if !ffmpeg_result.status.success() {
                let stderr = String::from_utf8_lossy(&ffmpeg_result.stderr);
                return Err(StreamingError::OtherError(format!("FFmpegの実行に失敗しました: {}", stderr)));
            }
            
            Ok(())
        }
        
        #[cfg(not(feature = "ffmpeg"))]
        {
            // FFmpeg機能が有効でない場合は単純な結合にフォールバック
            self.concat_segments(segment_files, output_file)
        }
    }
    
    /// セグメントを単純に結合します
    fn concat_segments(
        &self,
        segment_files: &[(PathBuf, f64)],
        output_file: &Path,
    ) -> Result<(), StreamingError> {
        let mut output = File::create(output_file)
            .map_err(|e| StreamingError::FileError(format!("出力ファイルの作成に失敗しました: {}", e)))?;
        
        for (file, _) in segment_files {
            let mut segment_data = fs::read(file)
                .map_err(|e| StreamingError::FileError(format!("セグメントの読み込みに失敗しました: {}", e)))?;
            
            output.write_all(&segment_data)
                .map_err(|e| StreamingError::FileError(format!("セグメントの書き込みに失敗しました: {}", e)))?;
        }
        
        Ok(())
    }
}

/// HLSセグメントをダウンロードする関数
async fn download_segment(
    client: &Client,
    url: &str,
    byte_range: Option<(u64, u64)>,
    output_path: &Path,
    timeout: Duration,
) -> Result<(), StreamingError> {
    // リクエストを構築
    let mut request = client.get(url);
    
    // バイトレンジが指定されている場合は、Rangeヘッダーを追加
    if let Some((start, end)) = byte_range {
        request = request.header("Range", format!("bytes={}-{}", start, end));
    }
    
    // タイムアウト設定
    request = request.timeout(timeout);
    
    // リクエストを送信
    let response = request.send()
        .await
        .map_err(|e| StreamingError::HttpError(format!("セグメントのダウンロードに失敗しました: {}", e)))?;
    
    // ステータスコードをチェック
    if !response.status().is_success() {
        return Err(StreamingError::HttpError(format!(
            "セグメントのダウンロードに失敗しました: ステータスコード {}", 
            response.status()
        )));
    }
    
    // レスポンスボディをファイルに書き込む
    let bytes = response.bytes()
        .await
        .map_err(|e| StreamingError::HttpError(e.to_string()))?;
    
    let mut file = File::create(output_path)
        .map_err(|e| StreamingError::FileError(format!("ファイルの作成に失敗しました: {}", e)))?;
    
    file.write_all(&bytes)
        .map_err(|e| StreamingError::FileError(format!("ファイルの書き込みに失敗しました: {}", e)))?;
    
    Ok(())
}

/// HLSマニフェスト
#[derive(Debug, Clone)]
pub enum HlsManifest {
    /// マスタープレイリスト
    Master(HlsMaster),
    /// メディアプレイリスト
    Media(HlsMedia),
}

/// HLSマスタープレイリスト
#[derive(Debug, Clone)]
pub struct HlsMaster {
    /// マスタープレイリストのURL
    pub url: String,
    /// バリアント（異なる品質の）プレイリスト
    pub variants: Vec<HlsVariant>,
}

/// HLSバリアント
#[derive(Debug, Clone)]
pub struct HlsVariant {
    /// バリアントプレイリストのURL
    pub url: String,
    /// 解像度（幅x高さ）
    pub resolution: Option<(u32, u32)>,
    /// ビットレート（bps）
    pub bandwidth: u64,
    /// コーデック情報
    pub codec: Option<String>,
}

/// HLSメディアプレイリスト
#[derive(Debug, Clone)]
pub struct HlsMedia {
    /// メディアプレイリストのURL
    pub url: String,
    /// メディアセグメント
    pub segments: Vec<StreamSegment>,
    /// ターゲット時間（秒）
    pub target_duration: f64,
    /// バージョン
    pub version: u8,
    /// シーケンス番号
    pub sequence_no: u64,
    /// ライブストリームかどうか
    pub is_live: bool,
}

impl Default for HlsDownloader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[tokio::test]
    async fn test_parse_master_playlist() {
        // マスタープレイリストのモックデータ
        let master_playlist = r#"#EXTM3U
#EXT-X-VERSION:3
#EXT-X-STREAM-INF:BANDWIDTH=1280000,RESOLUTION=640x360
http://example.com/low.m3u8
#EXT-X-STREAM-INF:BANDWIDTH=2560000,RESOLUTION=854x480
http://example.com/medium.m3u8
#EXT-X-STREAM-INF:BANDWIDTH=7680000,RESOLUTION=1920x1080
http://example.com/high.m3u8"#;
        
        // HTTPクライアントとURLのモック
        let url = "http://example.com/master.m3u8";
        
        // テストサーバーとモックの代わりにVecから直接解析
        let playlist = m3u8_rs::parse_playlist(master_playlist.as_bytes())
            .unwrap().1;
        
        match playlist {
            Playlist::MasterPlaylist(master) => {
                assert_eq!(master.variants.len(), 3);
                assert_eq!(master.variants[0].bandwidth, 1280000);
                assert_eq!(master.variants[1].bandwidth, 2560000);
                assert_eq!(master.variants[2].bandwidth, 7680000);
            },
            _ => panic!("Expected master playlist"),
        }
    }
    
    #[tokio::test]
    async fn test_parse_media_playlist() {
        // メディアプレイリストのモックデータ
        let media_playlist = r#"#EXTM3U
#EXT-X-VERSION:3
#EXT-X-TARGETDURATION:10
#EXT-X-MEDIA-SEQUENCE:0
#EXTINF:9.009,
segment1.ts
#EXTINF:9.009,
segment2.ts
#EXTINF:9.009,
segment3.ts
#EXTINF:9.009,
segment4.ts
#EXT-X-ENDLIST"#;
        
        // HTTPクライアントとURLのモック
        let url = "http://example.com/playlist.m3u8";
        
        // テストサーバーとモックの代わりにVecから直接解析
        let playlist = m3u8_rs::parse_playlist(media_playlist.as_bytes())
            .unwrap().1;
        
        match playlist {
            Playlist::MediaPlaylist(media) => {
                assert_eq!(media.segments.len(), 4);
                assert_eq!(media.target_duration, 10.0);
                assert_eq!(media.version, 3);
                assert_eq!(media.media_sequence, 0);
                assert!(media.end_list.is_some());
            },
            _ => panic!("Expected media playlist"),
        }
    }
}