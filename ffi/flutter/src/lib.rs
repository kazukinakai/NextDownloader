use anyhow::{anyhow, Result};
use flutter_rust_bridge::StreamSink;
use next_downloader_core::{
    content_type::ContentType,
    download::{DownloadManager, DownloadOptions, DownloadProgress, DownloadStatus},
    settings::Settings,
};
use std::sync::Arc;

/// ダウンロードマネージャーのラッパー
pub struct DownloaderApi {
    download_manager: Arc<DownloadManager>,
    settings: Arc<Settings>,
}

impl Default for DownloaderApi {
    fn default() -> Self {
        Self {
            download_manager: Arc::new(DownloadManager::new()),
            settings: Arc::new(Settings::default()),
        }
    }
}

impl DownloaderApi {
    /// 新しいDownloaderApiインスタンスを作成
    pub fn new() -> Self {
        Self::default()
    }

    /// URLからコンテンツをダウンロード
    pub async fn download_url(&self, url: String, output_path: String) -> Result<String> {
        let options = DownloadOptions {
            url,
            output_path: output_path.clone(),
            ..Default::default()
        };

        let download_id = self
            .download_manager
            .start_download(options)
            .await
            .map_err(|e| anyhow!("ダウンロード開始エラー: {}", e))?;

        Ok(download_id)
    }

    /// ダウンロードの進捗状況をストリームとして取得
    pub async fn get_download_progress(
        &self,
        download_id: String,
        sink: StreamSink<DownloadProgressInfo>,
    ) -> Result<()> {
        let mut progress_stream = self
            .download_manager
            .get_progress_stream(&download_id)
            .await
            .map_err(|e| anyhow!("進捗ストリーム取得エラー: {}", e))?;

        while let Some(progress) = progress_stream.recv().await {
            let progress_info = DownloadProgressInfo {
                download_id: download_id.clone(),
                bytes_downloaded: progress.bytes_downloaded,
                total_bytes: progress.total_bytes,
                percentage: progress.percentage,
                speed: progress.speed,
                status: match progress.status {
                    DownloadStatus::Pending => "pending".to_string(),
                    DownloadStatus::Downloading => "downloading".to_string(),
                    DownloadStatus::Completed => "completed".to_string(),
                    DownloadStatus::Failed => "failed".to_string(),
                    DownloadStatus::Cancelled => "cancelled".to_string(),
                    DownloadStatus::Paused => "paused".to_string(),
                },
                error_message: progress.error_message.unwrap_or_default(),
            };

            sink.add(progress_info);

            if matches!(
                progress.status,
                DownloadStatus::Completed | DownloadStatus::Failed | DownloadStatus::Cancelled
            ) {
                break;
            }
        }

        Ok(())
    }

    /// ダウンロードをキャンセル
    pub async fn cancel_download(&self, download_id: String) -> Result<()> {
        self.download_manager
            .cancel_download(&download_id)
            .await
            .map_err(|e| anyhow!("ダウンロードキャンセルエラー: {}", e))
    }

    /// ダウンロードを一時停止
    pub async fn pause_download(&self, download_id: String) -> Result<()> {
        self.download_manager
            .pause_download(&download_id)
            .await
            .map_err(|e| anyhow!("ダウンロード一時停止エラー: {}", e))
    }

    /// ダウンロードを再開
    pub async fn resume_download(&self, download_id: String) -> Result<()> {
        self.download_manager
            .resume_download(&download_id)
            .await
            .map_err(|e| anyhow!("ダウンロード再開エラー: {}", e))
    }

    /// アクティブなダウンロードの一覧を取得
    pub async fn get_active_downloads(&self) -> Result<Vec<DownloadInfo>> {
        let downloads = self
            .download_manager
            .get_active_downloads()
            .await
            .map_err(|e| anyhow!("アクティブダウンロード取得エラー: {}", e))?;

        let mut download_infos = Vec::new();
        for (id, progress) in downloads {
            download_infos.push(DownloadInfo {
                download_id: id,
                url: progress.url,
                output_path: progress.output_path,
                bytes_downloaded: progress.bytes_downloaded,
                total_bytes: progress.total_bytes,
                percentage: progress.percentage,
                speed: progress.speed,
                status: match progress.status {
                    DownloadStatus::Pending => "pending".to_string(),
                    DownloadStatus::Downloading => "downloading".to_string(),
                    DownloadStatus::Completed => "completed".to_string(),
                    DownloadStatus::Failed => "failed".to_string(),
                    DownloadStatus::Cancelled => "cancelled".to_string(),
                    DownloadStatus::Paused => "paused".to_string(),
                },
                error_message: progress.error_message.unwrap_or_default(),
            });
        }

        Ok(download_infos)
    }

    /// 設定を取得
    pub fn get_settings(&self) -> SettingsInfo {
        SettingsInfo {
            download_directory: self.settings.download_directory.clone(),
            max_concurrent_downloads: self.settings.max_concurrent_downloads,
            auto_detect_content_type: self.settings.auto_detect_content_type,
        }
    }

    /// 設定を更新
    pub fn update_settings(
        &mut self,
        download_directory: String,
        max_concurrent_downloads: u32,
        auto_detect_content_type: bool,
    ) -> Result<()> {
        let new_settings = Settings {
            download_directory,
            max_concurrent_downloads,
            auto_detect_content_type,
        };

        // 実際のアプリケーションでは、設定の保存処理も行う
        self.settings = Arc::new(new_settings);
        Ok(())
    }
}

/// ダウンロード情報
#[derive(Debug, Clone)]
pub struct DownloadInfo {
    pub download_id: String,
    pub url: String,
    pub output_path: String,
    pub bytes_downloaded: u64,
    pub total_bytes: Option<u64>,
    pub percentage: f64,
    pub speed: f64,
    pub status: String,
    pub error_message: String,
}

/// ダウンロード進捗情報
#[derive(Debug, Clone)]
pub struct DownloadProgressInfo {
    pub download_id: String,
    pub bytes_downloaded: u64,
    pub total_bytes: Option<u64>,
    pub percentage: f64,
    pub speed: f64,
    pub status: String,
    pub error_message: String,
}

/// 設定情報
#[derive(Debug, Clone)]
pub struct SettingsInfo {
    pub download_directory: String,
    pub max_concurrent_downloads: u32,
    pub auto_detect_content_type: bool,
}