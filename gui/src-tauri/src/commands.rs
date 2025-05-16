use nextdownloader_core::{
    DownloadManager, 
    Downloader, 
    DownloadOptions, 
    ContentType,
    VideoFormat,
    SystemStatus,
    ProgressInfo
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;
use tauri::Manager;

// 進捗情報をフロントエンドに送信するための型
#[derive(Serialize, Clone)]
struct ProgressEvent {
    task_id: String,
    progress: f64,
    speed: String,
    eta: String,
}

// システム状態情報
#[derive(Serialize)]
struct SystemStatusInfo {
    ready: bool,
    ytdlp: bool,
    aria2c: bool,
    ffmpeg: bool,
    description: String,
}

// フロントエンドからのダウンロードリクエスト
#[derive(Deserialize)]
pub struct DownloadRequest {
    url: String,
    output_path: String,
    filename: String,
    format: String,
    connections: Option<u32>,
    splits: Option<u32>,
    chunk_size: Option<u32>,
}

// ダウンロード結果
#[derive(Serialize)]
pub struct DownloadResult {
    success: bool,
    file_path: String,
    message: String,
}

// コンテンツタイプ検出結果
#[derive(Serialize)]
pub struct ContentTypeResult {
    success: bool,
    content_type: String,
    message: String,
}

/// URLからダウンロード
#[tauri::command]
pub async fn download_url(
    app: tauri::AppHandle,
    request: DownloadRequest
) -> Result<DownloadResult, String> {
    let downloader = DownloadManager::new();
    
    // フォーマット変換
    let format = match request.format.to_lowercase().as_str() {
        "mp4" => VideoFormat::Mp4,
        "mkv" => VideoFormat::Mkv,
        "mp3" => VideoFormat::Mp3,
        _ => VideoFormat::Mp4,
    };
    
    // ダウンロードオプション
    let options = DownloadOptions {
        connections: request.connections.unwrap_or(16),
        splits: request.splits.unwrap_or(16),
        chunk_size: request.chunk_size.unwrap_or(4),
        format,
        ..Default::default()
    };
    
    // タスクID生成
    let task_id = uuid::Uuid::new_v4().to_string();
    let task_id_clone = task_id.clone();
    
    // 進捗コールバック
    let app_handle = Arc::new(app);
    let app_handle_clone = Arc::clone(&app_handle);
    let progress_callback = Box::new(move |info: ProgressInfo| {
        // フロントエンドにイベント送信
        let _ = app_handle_clone.emit_all("download-progress", ProgressEvent {
            task_id: task_id_clone.clone(),
            progress: info.progress,
            speed: info.speed.clone(),
            eta: info.eta.clone(),
        });
    });
    
    // ダウンロード実行
    let output_path = PathBuf::from(&request.output_path);
    match downloader.download(
        &request.url,
        &output_path,
        &request.filename,
        Some(options),
        Some(progress_callback)
    ).await {
        Ok(file_path) => {
            // 完了イベント送信
            let _ = app_handle.emit_all("download-finished", ProgressEvent {
                task_id: task_id.clone(),
                progress: 1.0,
                speed: "完了".to_string(),
                eta: "0s".to_string(),
            });
            
            Ok(DownloadResult {
                success: true,
                file_path: file_path.to_string_lossy().to_string(),
                message: "ダウンロードが完了しました".to_string(),
            })
        },
        Err(err) => {
            // エラーイベント送信
            let _ = app_handle.emit_all("download-error", ProgressEvent {
                task_id,
                progress: 0.0,
                speed: "エラー".to_string(),
                eta: "失敗".to_string(),
            });
            
            Err(err.to_string())
        }
    }
}

/// コンテンツタイプを検出
#[tauri::command]
pub async fn detect_content_type(url: String) -> Result<ContentTypeResult, String> {
    let downloader = DownloadManager::new();
    
    match downloader.detect_content_type(&url).await {
        Ok(content_type) => {
            let type_str = match content_type {
                ContentType::Mp4 => "mp4",
                ContentType::Hls => "hls",
                ContentType::Dash => "dash",
                ContentType::YouTube => "youtube",
                ContentType::Unknown => "unknown",
            };
            
            Ok(ContentTypeResult {
                success: true,
                content_type: type_str.to_string(),
                message: "コンテンツタイプの検出に成功しました".to_string(),
            })
        },
        Err(err) => {
            Err(err.to_string())
        }
    }
}

/// システム状態をチェック
#[tauri::command]
pub async fn check_system_status() -> Result<SystemStatusInfo, String> {
    let downloader = DownloadManager::new();
    let (ytdlp, aria2c, ffmpeg) = downloader.check_dependencies().await;
    
    let status = SystemStatus::MissingDependencies {
        ytdlp,
        aria2c,
        ffmpeg,
    };
    
    Ok(SystemStatusInfo {
        ready: ytdlp && aria2c && ffmpeg,
        ytdlp,
        aria2c,
        ffmpeg,
        description: status.description(),
    })
}
