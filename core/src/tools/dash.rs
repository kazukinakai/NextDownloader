use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::collections::HashMap;
use std::fs;
use tokio::sync::Mutex;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use futures::{stream, StreamExt};
use reqwest::{Client, Url, header};
use anyhow::Result;
use quick_xml::de::from_str;
use serde::{Deserialize, Serialize};
use serde_json;
use crate::types::{DownloadError, ProgressCallback, DownloadOptions, ProgressInfo};
use crate::tools::{YtDlpTool, Aria2cTool, FFmpegTool};
use crate::utils::ensure_dir_exists;

/// ダウンロードの状態を表す構造体
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DownloadState {
    /// ダウンロード元のURL
    pub source_url: String,
    /// 出力先パス
    pub output_path: String,
    /// ダウンロードが完了したセグメントのインデックス
    pub completed_segments: Vec<usize>,
    /// セグメントの総数
    pub total_segments: usize,
    /// セグメントのURLと対応するファイルパスのマッピング
    pub segment_mapping: HashMap<String, String>,
    /// ダウンロードオプション
    pub options: DownloadOptions,
    /// 最終更新日時
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl DownloadState {
    /// 新しいダウンロード状態を作成
    pub fn new(
        source_url: String,
        output_path: String,
        total_segments: usize,
        segment_mapping: HashMap<String, String>,
        options: DownloadOptions,
    ) -> Self {
        Self {
            source_url,
            output_path,
            completed_segments: Vec::new(),
            total_segments,
            segment_mapping,
            options,
            updated_at: chrono::Utc::now(),
        }
    }

    /// 状態をファイルに保存
    pub async fn save(&self, path: &Path) -> Result<(), DownloadError> {
        let parent = path.parent().ok_or_else(|| DownloadError::InvalidInput("Invalid state file path".to_string()))?;
        ensure_dir_exists(parent)?;
        
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| DownloadError::SerializationError(e.to_string()))?;
            
        tokio::fs::write(path, json).await?;
        Ok(())
    }

    /// 状態をファイルから読み込み
    pub async fn load(path: &Path) -> Result<Self, DownloadError> {
        let content = tokio::fs::read_to_string(path).await?;
        let state: Self = serde_json::from_str(&content)
            .map_err(|e| DownloadError::DeserializationError(e.to_string()))?;
        Ok(state)
    }

    /// セグメントが完了したかどうかをチェック
    pub fn is_segment_completed(&self, index: usize) -> bool {
        self.completed_segments.contains(&index)
    }

    /// セグメントを完了としてマーク
    pub fn mark_segment_completed(&mut self, index: usize) {
        if !self.completed_segments.contains(&index) {
            self.completed_segments.push(index);
            self.updated_at = chrono::Utc::now();
        }
    }

    /// 進捗を計算（0.0 〜 1.0）
    pub fn progress(&self) -> f64 {
        if self.total_segments == 0 {
            return 0.0;
        }
        self.completed_segments.len() as f64 / self.total_segments as f64
    }
}

/// 状態ファイルのパスを取得
fn get_state_file_path(output_path: &Path) -> PathBuf {
    let mut state_path = output_path.to_path_buf();
    state_path.set_extension("json");
    state_path
}

/// DASHダウンロードを扱うための構造体
pub struct DashDownloadTool {
    ytdlp: YtDlpTool,
    aria2c: Aria2cTool,
    ffmpeg: FFmpegTool,
    client: Client,
    active_downloads: Arc<Mutex<std::collections::HashMap<String, bool>>>,
}

/// DASH MPDのルート要素
#[derive(Debug, Deserialize, Serialize)]
struct MPD {
    #[serde(rename = "Period")]
    periods: Vec<Period>,
    #[serde(rename = "BaseURL")]
    #[serde(default)]
    base_url: Option<String>,
}

/// DASH MPDのPeriod要素
#[derive(Debug, Deserialize, Serialize)]
struct Period {
    #[serde(rename = "AdaptationSet")]
    adaptation_sets: Vec<AdaptationSet>,
}

/// DASH MPDのAdaptationSet要素
#[derive(Debug, Deserialize, Serialize)]
struct AdaptationSet {
    #[serde(rename = "contentType")]
    #[serde(default)]
    content_type: Option<String>,
    #[serde(rename = "mimeType")]
    #[serde(default)]
    mime_type: Option<String>,
    #[serde(rename = "Representation")]
    representations: Vec<Representation>,
    #[serde(rename = "SegmentTemplate")]
    #[serde(default)]
    segment_template: Option<SegmentTemplate>,
}

/// DASH MPDのRepresentation要素
#[derive(Debug, Deserialize, Serialize)]
struct Representation {
    id: String,
    #[serde(rename = "bandwidth")]
    #[serde(default)]
    bandwidth: Option<u64>,
    #[serde(rename = "width")]
    #[serde(default)]
    width: Option<u32>,
    #[serde(rename = "height")]
    #[serde(default)]
    height: Option<u32>,
    #[serde(rename = "BaseURL")]
    #[serde(default)]
    base_url: Option<String>,
    #[serde(rename = "SegmentTemplate")]
    #[serde(default)]
    segment_template: Option<SegmentTemplate>,
    #[serde(rename = "SegmentList")]
    #[serde(default)]
    segment_list: Option<SegmentList>,
}

/// DASH MPDのSegmentTemplate要素
#[derive(Debug, Deserialize, Serialize)]
struct SegmentTemplate {
    #[serde(rename = "initialization")]
    #[serde(default)]
    initialization: Option<String>,
    #[serde(rename = "media")]
    #[serde(default)]
    media: Option<String>,
    #[serde(rename = "startNumber")]
    #[serde(default)]
    start_number: Option<u32>,
    #[serde(rename = "duration")]
    #[serde(default)]
    duration: Option<u32>,
    #[serde(rename = "timescale")]
    #[serde(default)]
    timescale: Option<u32>,
    #[serde(rename = "SegmentTimeline")]
    #[serde(default)]
    segment_timeline: Option<SegmentTimeline>,
}

/// DASH MPDのSegmentList要素
#[derive(Debug, Deserialize, Serialize)]
struct SegmentList {
    #[serde(rename = "SegmentURL")]
    segment_urls: Vec<SegmentURL>,
}

/// DASH MPDのSegmentURL要素
#[derive(Debug, Deserialize, Serialize)]
struct SegmentURL {
    #[serde(rename = "media")]
    media: String,
}

/// DASH MPDのSegmentTimeline要素
#[derive(Debug, Deserialize, Serialize)]
struct SegmentTimeline {
    #[serde(rename = "S")]
    segments: Vec<TimelineSegment>,
}

/// DASH MPDのS要素（タイムラインセグメント）
#[derive(Debug, Deserialize, Serialize)]
struct TimelineSegment {
    #[serde(rename = "t")]
    #[serde(default)]
    time: Option<u64>,
    #[serde(rename = "d")]
    duration: u64,
    #[serde(rename = "r")]
    #[serde(default)]
    repeat: Option<i32>,
}

impl DashDownloadTool {
    /// 並列でセグメントをダウンロード（レジューム対応）
    pub async fn download_segments_parallel(
        &self,
        segment_urls: &[String],
        output_dir: &Path,
        max_concurrent: usize,
        progress_callback: Option<ProgressCallback>,
        state: Option<&mut DownloadState>,
    ) -> Result<Vec<PathBuf>, DownloadError> {
        ensure_dir_exists(output_dir)?;
        let client = Client::new();
        
        // ダウンロードタスクを格納するベクター
        let mut tasks = Vec::with_capacity(segment_urls.len());
        let mut output_paths = Vec::with_capacity(segment_urls.len());
        let mut completed_segments = 0;
        
        // 各セグメントのダウンロードタスクを作成
        for (index, url) in segment_urls.iter().enumerate() {
            let output_path = output_dir.join(format!("segment_{:04}.m4s", index));
            
            // 既に完了しているセグメントはスキップ
            if let Some(state) = state.as_ref() {
                if state.is_segment_completed(index) {
                    completed_segments += 1;
                    output_paths.push(output_path);
                    continue;
                }
            }
            
            let url = url.clone();
            let client = client.clone();
            let active_downloads = self.active_downloads.clone();
            
            // 進捗コールバックを設定
            let progress_cb = progress_callback.clone();
            let state_clone = state.cloned();
            
            let task = tokio::spawn(async move {
                // ダウンロード中としてマーク
                let download_id = format!("{}:{}", url, output_path.display());
                {
                    let mut active = active_downloads.lock().await;
                    active.insert(download_id.clone(), true);
                }
                
                // セグメントをダウンロード
                let result = {
                    let client = client.clone();
                    let output_path = output_path.clone();
                    async move {
                        let result = Self::download_segment(&client, &url, &output_path).await;
                        
                        // 進捗を通知
                        if let Some(ref cb) = &progress_cb {
                            let progress = ProgressInfo {
                                total: 1,
                                downloaded: if result.is_ok() { 1 } else { 0 },
                                failed: if result.is_err() { 1 } else { 0 },
                                message: Some(format!("Downloaded segment {}", index + 1)),
                            };
                            cb(progress);
                        }
                        
                        // 状態を更新
                        if result.is_ok() {
                            if let Some(state) = &state_clone {
                                let mut state = state.lock().await;
                                state.mark_segment_completed(index);
                                if let Err(e) = state.save(&get_state_file_path(Path::new(&state.output_path))).await {
                                    log::error!("Failed to save download state: {}", e);
                                }
                            }
                        }
                        
                        result
                    }
                }.await;
                
                // ダウンロード完了としてマーク
                {
                    let mut active = active_downloads.lock().await;
                    active.remove(&download_id);
                }
                
                result.map(|_| output_path)
            });
            
            tasks.push(task);
            output_paths.push(output_path);
        }
        
        // 進捗を初期化
        if let Some(cb) = &progress_callback {
            cb(ProgressInfo {
                total: segment_urls.len() as u64,
                downloaded: completed_segments as u64,
                failed: 0,
                message: Some(format!("Resuming download: {}/{} segments", completed_segments, segment_urls.len())),
            });
        }
        
        // 並列ダウンロードを実行（最大同時ダウンロード数制御）
        let stream = futures::stream::iter(tasks)
            .buffer_unordered(max_concurrent);
            
        let results: Vec<Result<PathBuf, DownloadError>> = stream.collect().await;
        
        // 結果をチェック
        let mut success_paths = Vec::with_capacity(segment_urls.len());
        let mut failed_segments = Vec::new();
        
        for (i, result) in results.into_iter().enumerate() {
            match result {
                Ok(path) => {
                    if path.exists() {
                        success_paths.push(path);
                    } else {
                        failed_segments.push(i);
                    }
                }
                Err(e) => {
                    log::error!("Failed to download segment {}: {}", i, e);
                    failed_segments.push(i);
                }
            }
        }
        
        // 失敗したセグメントがある場合はエラーを返す
        if !failed_segments.is_empty() {
            return Err(DownloadError::NetworkError(
                format!("Failed to download {} segments", failed_segments.len())
            ));
        }
        
        Ok(success_paths)
    }
    /// ダウンロードしたセグメントを結合して1つのファイルに
    pub async fn merge_segments(
        &self,
        segment_paths: &[PathBuf],
        output_path: &Path,
        progress_callback: Option<ProgressCallback>,
    ) -> Result<(), DownloadError> {
        if segment_paths.is_empty() {
            return Err(DownloadError::InvalidInput("No segments to merge".to_string()));
        }
        
        // 出力ディレクトリが存在することを確認
        if let Some(parent) = output_path.parent() {
            ensure_dir_exists(parent)?;
        }
        
        // 一時ファイルにセグメントのリストを書き出し
        let temp_list = tempfile::Builder::new()
            .suffix(".txt")
            .tempfile_in(".")
            .map_err(|e| DownloadError::IoError(e))?;
            
        // セグメントのパスを一時ファイルに書き出し
        let temp_path = temp_list.path().to_owned();
        {
            let mut file = tokio::fs::File::create(&temp_path).await?;
            for path in segment_paths {
                let line = format!("file '{}'\n", path.display().to_string().replace("'", "'\\''"));
                file.write_all(line.as_bytes()).await?;
            }
        }
        
        // FFmpegで結合
        let args = [
            "-f", "concat",
            "-safe", "0",
            "-i", temp_path.to_str().ok_or_else(|| DownloadError::InvalidInput("Invalid temp path".to_string()))?,
            "-c", "copy",
            output_path.to_str().ok_or_else(|| DownloadError::InvalidInput("Invalid output path".to_string()))?,
        ];
        
        self.ffmpeg.execute(&args, progress_callback).await?;
        
        // 一時ファイルを削除
        drop(temp_list);
        
        // 結合されたファイルが存在するか確認
        if !output_path.exists() {
            return Err(DownloadError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Merged file not found: {}", output_path.display()),
            )));
        }
        
        Ok(())
    }
    
    /// DASHストリームをダウンロード（レジューム対応）
    pub async fn download_dash(
        &self,
        mpd_url: &str,
        output_path: &Path,
        options: &DownloadOptions,
        progress_callback: Option<ProgressCallback>,
    ) -> Result<(), DownloadError> {
        // 状態ファイルのパスを取得
        let state_file = get_state_file_path(output_path);
        let mut state = if state_file.exists() {
            // 状態ファイルが存在する場合は読み込む
            let mut state = DownloadState::load(&state_file).await?;
            
            // 同じURLからのダウンロードでなければ、状態をリセット
            if state.source_url != mpd_url || state.output_path != output_path.to_string_lossy() {
                state = DownloadState::new(
                    mpd_url.to_string(),
                    output_path.to_string_lossy().to_string(),
                    0,
                    HashMap::new(),
                    options.clone(),
                );
            }
            
            Some(Arc::new(tokio::sync::Mutex::new(state)))
        } else {
            // 状態ファイルが存在しない場合は新規作成
            None
        };
        
        // 新しい状態を作成
        if state.is_none() {
            state = Some(Arc::new(tokio::sync::Mutex::new(DownloadState::new(
                mpd_url.to_string(),
                output_path.to_string_lossy().to_string(),
                0,
                HashMap::new(),
                options.clone(),
            ))));
        }
        // MPDを解析
        let mpd = self.parse_mpd(mpd_url).await?;
        
        if mpd.periods.is_empty() {
            return Err(DownloadError::InvalidInput("No periods found in MPD".to_string()));
        }
        
        // 最初のピリオドを使用
        let period = &mpd.periods[0];
        
        // 最適な品質のRepresentationを選択
        let (video_rep, audio_rep) = self.select_best_representation(&period.adaptation_sets, options);
        
        if video_rep.is_none() && audio_rep.is_none() {
            return Err(DownloadError::InvalidInput("No suitable representations found".to_string()));
        }
        
        // 一時ディレクトリを作成
        let temp_dir = tempfile::tempdir()?;
        let video_dir = temp_dir.path().join("video");
        let audio_dir = temp_dir.path().join("audio");
        let output_dir = temp_dir.path().join("output");
        
        tokio::fs::create_dir_all(&video_dir).await?;
        tokio::fs::create_dir_all(&audio_dir).await?;
        tokio::fs::create_dir_all(&output_dir).await?;
        
        // ベースURLを取得
        let base_url = self.get_base_url(mpd_url);
        
        // ビデオとオーディオのセグメントをダウンロード
        let mut video_paths = Vec::new();
        let mut audio_paths = Vec::new();
        
        if let Some((video_rep, adaptation_set)) = video_rep.and_then(|rep| {
            period.adaptation_sets.iter()
                .find(|a| a.representations.iter().any(|r| r.id == rep.id))
                .map(|a| (rep, a))
        }) {
            // ビデオセグメントのURLを生成
            let video_segment_urls = self.generate_segment_urls(&base_url, video_rep, adaptation_set);
            
            // 並列でダウンロード（レジューム対応）
            video_paths = self.download_segments_parallel(
                &video_segment_urls,
                &video_dir,
                options.max_concurrent_downloads.unwrap_or(4) as usize,
                progress_callback.clone(),
                state.as_deref_mut(),
            ).await?;
            
            // 一時的にビデオを結合
            let video_temp = output_dir.join("video_temp.mp4");
            self.merge_segments(&video_paths, &video_temp, progress_callback.clone()).await?;
        }
        
        if let Some((audio_rep, adaptation_set)) = audio_rep.and_then(|rep| {
            period.adaptation_sets.iter()
                .find(|a| a.representations.iter().any(|r| r.id == rep.id))
                .map(|a| (rep, a))
        }) {
            // オーディオセグメントのURLを生成
            let audio_segment_urls = self.generate_segment_urls(&base_url, audio_rep, adaptation_set);
            
            // 並列でダウンロード（レジューム対応）
            audio_paths = self.download_segments_parallel(
                &audio_segment_urls,
                &audio_dir,
                options.max_concurrent_downloads.unwrap_or(4) as usize,
                progress_callback.clone(),
                state.as_deref_mut(),
            ).await?;
            
            // 一時的にオーディオを結合
            let audio_temp = output_dir.join("audio_temp.m4a");
            self.merge_segments(&audio_paths, &audio_temp, progress_callback.clone()).await?;
        }
        
        // ビデオとオーディオを結合
        if !video_paths.is_empty() && !audio_paths.is_empty() {
            // 両方のストリームがある場合は結合
            let video_temp = output_dir.join("video_temp.mp4");
            let audio_temp = output_dir.join("audio_temp.m4a");
            
            let args = [
                "-i", video_temp.to_str().unwrap(),
                "-i", audio_temp.to_str().unwrap(),
                "-c:v", "copy",
                "-c:a", "aac",
                "-strict", "experimental",
                output_path.to_str().ok_or_else(|| DownloadError::InvalidInput("Invalid output path".to_string()))?,
            ];
            
            self.ffmpeg.execute(&args, progress_callback).await?;
        } else if !video_paths.is_empty() {
            // ビデオのみ
            let video_temp = output_dir.join("video_temp.mp4");
            tokio::fs::copy(&video_temp, output_path).await?;
        } else if !audio_paths.is_empty() {
            // オーディオのみ
            let audio_temp = output_dir.join("audio_temp.m4a");
            tokio::fs::copy(&audio_temp, output_path).await?;
        }
        
        // ダウンロードが完了したら状態ファイルを削除
        if let Some(state) = state {
            let state_path = get_state_file_path(output_path);
            if state_path.exists() {
                tokio::fs::remove_file(state_path).await?;
            }
        }
        
        // 一時ファイルを削除
        drop(temp_dir);
        
        Ok(())
    }
    
    /// 新しいDashDownloadToolを作成
    pub fn new() -> Self {
        Self {
            ytdlp: YtDlpTool::new(),
            aria2c: Aria2cTool::new(),
            ffmpeg: FFmpegTool::new(),
            client: Client::builder()
                .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/123.0.0.0 Safari/537.36")
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
            active_downloads: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }
    
    /// DASHマニフェスト(MPD)を解析
    pub async fn parse_mpd(&self, url: &str) -> Result<MPD, DownloadError> {
        // まず、URLがmpdファイルを直接指しているか確認
        if url.ends_with(".mpd") {
            return self.parse_mpd_directly(url).await;
        }
        
        // yt-dlpを使用してURLを展開
        let result = tokio::process::Command::new(self.ytdlp.executable_path())
            .args(&["--get-url", "--no-warnings", url])
            .output()
            .await
            .map_err(|e| DownloadError::ProcessFailed(format!("yt-dlpの実行に失敗しました: {}", e)))?;
            
        if !result.status.success() {
            let error_message = String::from_utf8_lossy(&result.stderr);
            return Err(DownloadError::ProcessFailed(format!("yt-dlpがエラーを返しました: {}", error_message)));
        }
        
        let output = String::from_utf8_lossy(&result.stdout);
        let urls: Vec<String> = output
            .lines()
            .filter(|line| !line.is_empty())
            .map(|line| line.to_string())
            .collect();
        
        // 取得したURLがmpdファイルを指している場合、そのマニフェストを解析
        for url in urls {
            if url.ends_with(".mpd") {
                return self.parse_mpd_directly(&url).await;
            }
        }
        
        // mpdが見つからない場合はエラー
        Err(DownloadError::ProcessFailed("DASHマニフェスト(MPD)が見つかりません".to_string()))
    }
    
    /// mpdファイルを直接解析
    async fn parse_mpd_directly(&self, url: &str) -> Result<MPD, DownloadError> {
        // mpdファイルを取得
        let response = self.client.get(url)
            .send()
            .await
            .map_err(|e| DownloadError::NetworkError(format!("MPDファイルの取得に失敗しました: {}", e)))?;
        
        if !response.status().is_success() {
            return Err(DownloadError::NetworkError(format!("MPDファイルの取得に失敗しました: HTTP {}", response.status())));
        }
        
        let content = response.text().await
            .map_err(|e| DownloadError::ProcessFailed(format!("MPDファイルの読み込みに失敗しました: {}", e)))?;
        
        // XMLをパース
        let mpd: MPD = from_str(&content)
            .map_err(|e| DownloadError::ProcessFailed(format!("MPDファイルの解析に失敗しました: {}", e)))?;
        
        Ok(mpd)
    }
    
    /// URLからベースURLを取得
    fn get_base_url(&self, url: &str) -> String {
        if let Ok(parsed_url) = Url::parse(url) {
            // URLの最後のスラッシュまでを取得
            let path = parsed_url.path();
            let last_slash_pos = path.rfind('/').unwrap_or(0);
            let base_path = &path[..=last_slash_pos];
            
            // ベースURLを再構築
            let mut base_url = parsed_url.clone();
            base_url.set_path(base_path);
            base_url.to_string()
        } else {
            // URLの解析に失敗した場合、最後のスラッシュまでを取得
            let last_slash_pos = url.rfind('/').unwrap_or(0);
            url[..=last_slash_pos].to_string()
        }
    }
    
    /// 最適な品質のRepresentationを選択
    fn select_best_representation(&self, adaptation_sets: &[AdaptationSet], options: &DownloadOptions) -> (Option<&Representation>, Option<&Representation>) {
        let mut best_video: Option<&Representation> = None;
        let mut best_audio: Option<&Representation> = None;
        
        // 解像度の優先順位を決定
        let target_height = options.video_height.unwrap_or(720);
        
        // ビデオとオーディオのAdaptationSetを探す
        for adaptation_set in adaptation_sets {
            let is_video = adaptation_set.content_type.as_deref() == Some("video") || 
                          adaptation_set.mime_type.as_deref().map_or(false, |m| m.starts_with("video/"));
            
            let is_audio = adaptation_set.content_type.as_deref() == Some("audio") || 
                          adaptation_set.mime_type.as_deref().map_or(false, |m| m.starts_with("audio/"));
            
            if is_video {
                // ビデオの場合、指定された解像度に最も近いものを選択
                for rep in &adaptation_set.representations {
                    if let Some(height) = rep.height {
                        if best_video.is_none() || 
                           (height <= target_height && height > best_video.unwrap().height.unwrap_or(0)) || 
                           (best_video.unwrap().height.unwrap_or(0) > target_height && height < best_video.unwrap().height.unwrap_or(0)) {
                            best_video = Some(rep);
                        }
                    }
                }
            } else if is_audio {
                // オーディオの場合、最高ビットレートのものを選択
                for rep in &adaptation_set.representations {
                    if let Some(bandwidth) = rep.bandwidth {
                        if best_audio.is_none() || bandwidth > best_audio.unwrap().bandwidth.unwrap_or(0) {
                            best_audio = Some(rep);
                        }
                    }
                }
            }
        }
        
        (best_video, best_audio)
    }
    
    /// セグメントURLのリストを生成
    fn generate_segment_urls(
        &self, 
        base_url: &str, 
        representation: &Representation, 
        adaptation_set: &AdaptationSet
    ) -> Vec<String> {
        let mut segment_urls = Vec::new();
        
        // SegmentListがある場合
        if let Some(segment_list) = &representation.segment_list {
            for segment_url in &segment_list.segment_urls {
                let url = if segment_url.media.starts_with("http") {
                    segment_url.media.clone()
                } else {
                    format!("{}{}", base_url, segment_url.media)
                };
                segment_urls.push(url);
            }
            return segment_urls;
        }
        
        // SegmentTemplateを取得（RepresentationまたはAdaptationSetから）
        let segment_template = representation.segment_template.as_ref()
            .or_else(|| adaptation_set.segment_template.as_ref());
        
        if let Some(template) = segment_template {
            // 初期化セグメントがあれば追加
            if let Some(init) = &template.initialization {
                let init_url = if init.starts_with("http") {
                    init.clone()
                } else {
                    let url = init.replace("$RepresentationID$", &representation.id);
                    format!("{}{}", base_url, url)
                };
                segment_urls.push(init_url);
            }
            
            // メディアセグメントのURLを生成
            if let Some(media) = &template.media {
                // SegmentTimelineがある場合
                if let Some(timeline) = &template.segment_timeline {
                    let mut current_number = template.start_number.unwrap_or(1);
                    
                    for s in &timeline.segments {
                        let repeat = s.repeat.unwrap_or(0) + 1;
                        
                        for _ in 0..repeat {
                            let url = media
                                .replace("$RepresentationID$", &representation.id)
                                .replace("$Number$", &current_number.to_string());
                            
                            let full_url = if url.starts_with("http") {
                                url
                            } else {
                                format!("{}{}", base_url, url)
                            };
                            
                            segment_urls.push(full_url);
                            current_number += 1;
                        }
                    }
                } 
                // durationとtimescaleがある場合（簡易的な実装）
                else if let (Some(duration), Some(timescale)) = (template.duration, template.timescale) {
                    let start = template.start_number.unwrap_or(1);
                    // 仮の終了番号（実際には動的に調整する必要がある）
                    let end = start + 100; // 100セグメントを仮定
                    
                    for number in start..end {
                        let url = media
                            .replace("$RepresentationID$", &representation.id)
                            .replace("$Number$", &number.to_string());
                        
                        let full_url = if url.starts_with("http") {
                            url
                        } else {
                            format!("{}{}", base_url, url)
                        };
                        
                        segment_urls.push(full_url);
                    }
                }
            }
        }
        
        segment_urls
    }
    
    /// 個別のセグメントをダウンロード（レジューム対応）
    async fn download_segment(
        &self,
        client: &Client,
        url: &str,
        output_path: &PathBuf,
    ) -> Result<(), DownloadError> {
        // 既存のファイルがあれば、そのサイズを取得してレジューム
        let mut file = match tokio::fs::File::open(output_path).await {
            Ok(file) => file,
            Err(_) => tokio::fs::File::create(output_path).await
                .map_err(|e| DownloadError::IoError(Box::new(e)))?,
        };

        let file_metadata = file.metadata().await
            .map_err(|e| DownloadError::IoError(Box::new(e)))?;
        let file_size = file_metadata.len();

        // レジューム用のリクエストを構築
        let mut request_builder = client.get(url);
        
        // ファイルが存在し、サイズが0より大きい場合はレジューム
        if file_size > 0 {
            request_builder = request_builder.header(
                header::RANGE,
                format!("bytes={}-", file_size)
            );
        }

        let mut response = request_builder
            .send()
            .await
            .map_err(|e| DownloadError::NetworkError(e.to_string()))?;

        // レジュームできない場合は、ファイルを削除して最初から再試行
        if response.status() == reqwest::StatusCode::RANGE_NOT_SATISFIABLE {
            tokio::fs::remove_file(output_path).await
                .map_err(|e| DownloadError::IoError(Box::new(e)))?;
            return self.download_segment(client, url, output_path).await;
        }

        if !response.status().is_success() {
            return Err(DownloadError::HttpError(response.status().as_u16()));
        }

        // レジュームの場合は追記モードで開き直す
        if file_size > 0 && response.status() == reqwest::StatusCode::PARTIAL_CONTENT {
            file = tokio::fs::OpenOptions::new()
                .append(true)
                .open(output_path)
                .await
                .map_err(|e| DownloadError::IoError(Box::new(e)))?;
        }

        // ストリームからデータを読み込んでファイルに書き込み
        while let Some(chunk) = response.chunk().await
            .map_err(|e| DownloadError::NetworkError(e.to_string()))? {
            file.write_all(&chunk).await
                .map_err(|e| DownloadError::IoError(Box::new(e)))?;
        }

        Ok(())
    }
}
