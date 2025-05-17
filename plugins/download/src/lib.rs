use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use std::marker::PhantomData;

use tauri::{
    plugin::{Builder, TauriPlugin},
    Runtime, State, Manager,
};

use nextdownloader_core::{
    DownloadManager, Downloader, DownloadOptions, ContentType, VideoFormat, ErrorCode
};

// プラグインのエラー型
#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("Invalid URL")]
    InvalidUrl,
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("File system error: {0}")]
    FileSystemError(String),
    #[error("Dependency error: {0}")]
    DependencyError(String),
    #[error("Unknown error: {0}")]
    UnknownError(String),
}

// Serdeシリアライズ用のコンテンツタイプ
#[derive(serde::Serialize)]
pub enum ContentTypeResponse {
    MP4,
    HLS,
    DASH,
    YouTube,
    Unknown,
}

// コンテンツタイプの変換
impl From<ContentType> for ContentTypeResponse {
    fn from(ct: ContentType) -> Self {
        match ct {
            ContentType::MP4 => ContentTypeResponse::MP4,
            ContentType::HLS => ContentTypeResponse::HLS,
            ContentType::DASH => ContentTypeResponse::DASH,
            ContentType::YouTube => ContentTypeResponse::YouTube,
            ContentType::Unknown => ContentTypeResponse::Unknown,
        }
    }
}

// 依存関係チェック結果
#[derive(serde::Serialize)]
pub struct DependencyStatus {
    pub ytdlp: bool,
    pub aria2c: bool,
    pub ffmpeg: bool,
}

// プラグインの状態
pub struct DownloadPlugin<R: Runtime> {
    download_manager: Arc<Mutex<DownloadManager>>,
    _phantom: PhantomData<R>,
}

impl<R: Runtime> DownloadPlugin<R> {
    pub fn new() -> Self {
        Self {
            download_manager: Arc::new(Mutex::new(DownloadManager::new())),
            _phantom: PhantomData,
        }
    }
}

// 依存関係のチェック
#[tauri::command]
async fn check_dependencies<R: Runtime>(
    plugin: State<'_, DownloadPlugin<R>>,
) -> Result<DependencyStatus, String> {
    let download_manager = plugin.download_manager.lock().unwrap();
    
    // 非同期で依存関係をチェック
    let (ytdlp, aria2c, ffmpeg) = download_manager.check_dependencies().await;
    
    Ok(DependencyStatus {
        ytdlp,
        aria2c,
        ffmpeg,
    })
}

// コンテンツタイプの検出
#[tauri::command]
async fn detect_content_type<R: Runtime>(
    url: String,
    plugin: State<'_, DownloadPlugin<R>>,
) -> Result<ContentTypeResponse, String> {
    let download_manager = plugin.download_manager.lock().unwrap();
    
    // URLの検証
    if url.is_empty() {
        return Err("Invalid URL".to_string());
    }
    
    // コンテンツタイプの検出
    match download_manager.detect_content_type(&url).await {
        Ok(content_type) => Ok(content_type.into()),
        Err(e) => Err(format!("Error detecting content type: {:?}", e)),
    }
}

// ダウンロードの開始
#[tauri::command]
async fn start_download<R: Runtime>(
    url: String,
    destination: String,
    format: Option<String>,
    plugin: State<'_, DownloadPlugin<R>>,
    app: tauri::AppHandle<R>,
) -> Result<String, String> {
    let download_manager = plugin.download_manager.lock().unwrap();
    
    // URLとパスの検証
    if url.is_empty() {
        return Err("Invalid URL".to_string());
    }
    
    let dest_path = PathBuf::from(destination);
    
    // オプションの設定
    let mut options = DownloadOptions::default();
    if let Some(fmt) = format {
        options.preferred_format = Some(fmt);
    }
    
    // ダウンロードの開始
    match download_manager.start_download(&url, &dest_path, options).await {
        Ok(download_id) => {
            // ダウンロードの進捗を監視し、イベントとして発行
            let download_id_clone = download_id.clone();
            let app_handle = app.clone();
            let manager = Arc::clone(&plugin.download_manager);
            
            // 別スレッドでダウンロード進捗を監視
            tauri::async_runtime::spawn(async move {
                let interval = std::time::Duration::from_secs(1);
                loop {
                    tokio::time::sleep(interval).await;
                    
                    let progress = {
                        let dm = manager.lock().unwrap();
                        match dm.get_download_progress(&download_id_clone).await {
                            Ok(p) => p,
                            Err(_) => break, // エラーが発生した場合は監視を終了
                        }
                    };
                    
                    // 進捗イベントを発行
                    let _ = app_handle.emit_all("download-progress", (download_id_clone.clone(), progress));
                    
                    // ダウンロードが完了した場合は監視を終了
                    if progress >= 1.0 {
                        let _ = app_handle.emit_all("download-complete", download_id_clone.clone());
                        break;
                    }
                }
            });
            
            Ok(download_id)
        },
        Err(e) => Err(format!("Error starting download: {:?}", e)),
    }
}

// ダウンロードの進捗取得
#[tauri::command]
async fn get_download_progress<R: Runtime>(
    download_id: String,
    plugin: State<'_, DownloadPlugin<R>>,
) -> Result<f64, String> {
    let download_manager = plugin.download_manager.lock().unwrap();
    
    // ダウンロードIDの検証
    if download_id.is_empty() {
        return Err("Invalid download ID".to_string());
    }
    
    // 進捗の取得
    match download_manager.get_download_progress(&download_id).await {
        Ok(progress) => Ok(progress),
        Err(e) => Err(format!("Error getting download progress: {:?}", e)),
    }
}

// ダウンロードのキャンセル
#[tauri::command]
async fn cancel_download<R: Runtime>(
    download_id: String,
    plugin: State<'_, DownloadPlugin<R>>,
) -> Result<(), String> {
    let download_manager = plugin.download_manager.lock().unwrap();
    
    // ダウンロードIDの検証
    if download_id.is_empty() {
        return Err("Invalid download ID".to_string());
    }
    
    // ダウンロードのキャンセル
    match download_manager.cancel_download(&download_id).await {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Error cancelling download: {:?}", e)),
    }
}

// プラグインの初期化
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("download")
        .setup(|app| {
            // プラグインの状態を初期化
            app.manage(DownloadPlugin::<R>::new());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            check_dependencies,
            detect_content_type,
            start_download,
            get_download_progress,
            cancel_download,
        ])
        .build()
}