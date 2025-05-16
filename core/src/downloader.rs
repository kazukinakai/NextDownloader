use std::path::PathBuf;
use async_trait::async_trait;
use crate::types::{ContentType, DownloadOptions, DownloadError, ProgressCallback};

/// ダウンローダーの基本的なインターフェースを定義するトレイト
#[async_trait]
pub trait Downloader {
    /// URLからコンテンツタイプを検出
    async fn detect_content_type(&self, url: &str) -> Result<ContentType, DownloadError>;
    
    /// コンテンツをダウンロード
    async fn download(
        &self, 
        url: &str, 
        output_path: &PathBuf, 
        filename: &str,
        options: Option<DownloadOptions>,
        progress_callback: Option<ProgressCallback>
    ) -> Result<PathBuf, DownloadError>;
    
    /// ダウンロードをキャンセル
    async fn cancel_download(&self, task_id: &str) -> Result<(), DownloadError>;
}

/// ダウンロードマネージャーの実装
pub struct DownloadManager {
    ytdlp: crate::tools::ytdlp::YtDlpTool,
    aria2c: crate::tools::aria2c::Aria2cTool,
    ffmpeg: crate::tools::ffmpeg::FFmpegTool,
    hls: crate::tools::hls::HlsDownloadTool,
    active_tasks: tokio::sync::Mutex<std::collections::HashMap<String, tokio::task::JoinHandle<()>>>,
}

impl DownloadManager {
    /// 新しいダウンロードマネージャーを作成
    pub fn new() -> Self {
        Self {
            ytdlp: crate::tools::ytdlp::YtDlpTool::new(),
            aria2c: crate::tools::aria2c::Aria2cTool::new(),
            ffmpeg: crate::tools::ffmpeg::FFmpegTool::new(),
            hls: crate::tools::hls::HlsDownloadTool::new(),
            active_tasks: tokio::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }
    
    /// 依存関係をチェック
    pub async fn check_dependencies(&self) -> (bool, bool, bool) {
        let (ytdlp_available, aria2c_available, ffmpeg_available) = tokio::join!(
            self.ytdlp.is_available(),
            self.aria2c.is_available(),
            self.ffmpeg.is_available()
        );
        
        (ytdlp_available, aria2c_available, ffmpeg_available)
    }
    
    /// システム状態を取得
    pub async fn system_status(&self) -> crate::types::SystemStatus {
        let (ytdlp, aria2c, ffmpeg) = self.check_dependencies().await;
        
        if ytdlp && aria2c && ffmpeg {
            crate::types::SystemStatus::Ready
        } else {
            crate::types::SystemStatus::MissingDependencies {
                ytdlp,
                aria2c,
                ffmpeg,
            }
        }
    }
}

#[async_trait]
impl Downloader for DownloadManager {
    async fn detect_content_type(&self, url: &str) -> Result<ContentType, DownloadError> {
        // URLの拡張子をチェック
        if url.to_lowercase().ends_with(".mp4") {
            return Ok(ContentType::Mp4);
        } else if url.to_lowercase().ends_with(".m3u8") {
            return Ok(ContentType::Hls);
        } else if url.to_lowercase().ends_with(".mpd") {
            return Ok(ContentType::Dash);
        }
        
        // YouTubeのURLをチェック
        if url.contains("youtube.com") || url.contains("youtu.be") {
            return Ok(ContentType::YouTube);
        }
        
        // yt-dlpを使用してコンテンツタイプを検出
        match self.ytdlp.get_video_info(url).await {
            Ok(info) => {
                if let Some(formats) = &info.formats {
                    for format in formats {
                        if let Some(url) = &format.url {
                            if url.contains(".m3u8") {
                                return Ok(ContentType::Hls);
                            } else if url.contains(".mpd") {
                                return Ok(ContentType::Dash);
                            }
                        }
                    }
                }
                
                // デフォルトはMP4として扱う
                Ok(ContentType::Mp4)
            }
            Err(_) => Ok(ContentType::Unknown),
        }
    }
    
    async fn download(
        &self, 
        url: &str, 
        output_path: &PathBuf, 
        filename: &str,
        options: Option<DownloadOptions>,
        progress_callback: Option<ProgressCallback>
    ) -> Result<PathBuf, DownloadError> {
        // コンテンツタイプを検出
        let content_type = self.detect_content_type(url).await?;
        
        // オプションが指定されていない場合は、コンテンツタイプに基づいて最適なオプションを使用
        let download_options = options.unwrap_or_else(|| {
            match content_type {
                ContentType::Mp4 => DownloadOptions::default(),
                ContentType::Hls => DownloadOptions {
                    chunk_size: 1,
                    retry_wait: 1,
                    max_retries: 10,
                    ..Default::default()
                },
                ContentType::Dash => DownloadOptions {
                    connections: 8,
                    splits: 8,
                    chunk_size: 1,
                    use_quic: true,
                    ..Default::default()
                },
                _ => DownloadOptions::default(),
            }
        });
        
        // コンテンツタイプに応じたダウンロード方法を選択
        match content_type {
            ContentType::Mp4 => {
                self.aria2c.download(url, output_path, filename, &download_options, progress_callback).await
            },
            ContentType::Hls => {
                self.hls.download(url, output_path, filename, &download_options, progress_callback).await
            },
            ContentType::Dash | ContentType::YouTube | ContentType::Unknown => {
                self.ytdlp.download(url, output_path, filename, &download_options, progress_callback).await
            },
        }
    }
    
    async fn cancel_download(&self, task_id: &str) -> Result<(), DownloadError> {
        let mut tasks = self.active_tasks.lock().await;
        if let Some(handle) = tasks.remove(task_id) {
            handle.abort();
            Ok(())
        } else {
            Err(DownloadError::Internal("タスクが見つかりません".to_string()))
        }
    }
}
