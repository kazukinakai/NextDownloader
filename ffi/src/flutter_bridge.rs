// flutter_bridge.rs
// Flutter-Rust連携のためのブリッジインターフェース
// flutter_rust_bridgeの形式に合わせて実装

use flutter_rust_bridge::*;
use nextdownloader_core::{DownloadManager, DownloadOptions, ContentType, DownloadStatus, ErrorCode};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock, Mutex};
use std::path::PathBuf;
use tokio::runtime::Runtime;
use lazy_static::lazy_static;
use anyhow::{Result, anyhow};
use log::{debug, error, info, warn};
use std::ffi::{c_char, CStr, CString};
use std::os::raw::c_void;
use std::ptr;
use chrono::{DateTime, Utc};
use uuid::Uuid;

// グローバルダウンロードマネージャーのインスタンス
lazy_static! {
    static ref DOWNLOAD_MANAGER: Arc<RwLock<DownloadManager>> = Arc::new(RwLock::new(DownloadManager::new()));
    static ref RUNTIME: Mutex<Runtime> = Mutex::new(Runtime::new().unwrap());
}

// ダウンロード進捗情報を表すデータ構造
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub download_id: String,
    pub progress: f64,
    pub downloaded_size: u64,
    pub total_size: Option<u64>,
    pub status: String,
    pub error_message: Option<String>,
}

// ダウンロード情報を表すデータ構造
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadInfo {
    pub download_id: String,
    pub url: String,
    pub destination: String,
    pub status: String,
    pub progress: f64,
    pub downloaded_size: u64,
    pub total_size: Option<u64>,
    pub created_at: String,
    pub updated_at: String,
    pub error_message: Option<String>,
}

// ダウンロードオプションを表すデータ構造
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadOptionsWrapper {
    pub headers: Option<Vec<Header>>,
    pub timeout: Option<u64>,
    pub retry_count: Option<u32>,
    pub follow_redirects: Option<bool>,
}

// HTTPヘッダーを表すデータ構造
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Header {
    pub name: String,
    pub value: String,
}

// 設定情報を表すデータ構造
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub default_download_path: String,
    pub max_concurrent_downloads: i32,
    pub use_system_notification: bool,
    pub auto_extract_archives: bool,
    pub theme: String,
}

// YouTube動画情報を表すデータ構造
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YoutubeVideoInfo {
    pub title: String,
    pub author: String,
    pub duration: i32,
    pub formats: Vec<YoutubeVideoFormat>,
}

// YouTube動画フォーマットを表すデータ構造
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YoutubeVideoFormat {
    pub quality: String,
    pub format: String,
    pub size: i32,
}

// C FFI関数のエクスポート

#[no_mangle]
pub extern "C" fn initialize_downloader() -> i8 {
    match init_logger() {
        Ok(_) => info!("ロガーが初期化されました"),
        Err(e) => eprintln!("ロガーの初期化に失敗しました: {}", e),
    }

    info!("ダウンローダーを初期化しています...");
    
    // 既にダウンロードマネージャーは初期化されている（lazy_staticで）
    1
}

#[no_mangle]
pub extern "C" fn start_download(
    url_ptr: *const c_char,
    destination_ptr: *const c_char,
    options_json_ptr: *const c_char,
) -> *mut c_char {
    let result = catch_unwind_result(|| {
        let url = unsafe { CStr::from_ptr(url_ptr).to_str()? };
        let destination = unsafe { CStr::from_ptr(destination_ptr).to_str()? };
        let options_json = unsafe { CStr::from_ptr(options_json_ptr).to_str()? };
        
        info!("ダウンロード開始: {} -> {}", url, destination);
        
        // ダウンロードを開始
        let manager = DOWNLOAD_MANAGER.write()
            .map_err(|e| anyhow!("ダウンロードマネージャーの取得に失敗しました: {}", e))?;
        
        let runtime = RUNTIME.lock()
            .map_err(|e| anyhow!("ランタイムの取得に失敗しました: {}", e))?;
            
        let result = runtime.block_on(async {
            // 引数の型を正しく合わせる
            // start_downloadは引数としてurl: &str, destination: &Path, format: Option<&str>を取る
            let dest_path = PathBuf::from(destination);
            manager.start_download(url, &dest_path, None).await
        });
        
        match result {
            Ok(download_id) => {
                // 現在はget_downloadメソッドが実装されていないため、直接ダウンロード情報を作成
                let now = Utc::now();
                let now_str = now.to_rfc3339();
                
                let download_info = DownloadInfo {
                    download_id: download_id.clone(),
                    url: url.to_string(),
                    destination: destination.to_string(),
                    status: status_to_string(DownloadStatus::Downloading),
                    progress: 0.0,
                    downloaded_size: 0,
                    total_size: None,
                    created_at: now_str.clone(),
                    updated_at: now_str,
                    error_message: None,
                };
                
                let json = serde_json::to_string(&download_info)?;
                Ok(json)
            },
            Err(e) => Err(anyhow!("ダウンロードの開始に失敗しました: {}", e)),
        }
    });
    
    result_to_c_string(result)
}

#[no_mangle]
pub extern "C" fn get_download_progress(download_id_ptr: *const c_char) -> *mut c_char {
    let result = catch_unwind_result(|| {
        let download_id = unsafe { CStr::from_ptr(download_id_ptr).to_str()? };
        
        let manager = DOWNLOAD_MANAGER.read()
            .map_err(|e| anyhow!("ダウンロードマネージャーの取得に失敗しました: {}", e))?;
        
        // 現在はget_downloadメソッドが実装されていないため、ダミーの進捗情報を返す
        let runtime = RUNTIME.lock()
            .map_err(|e| anyhow!("ランタイムの取得に失敗しました: {}", e))?;
            
        let progress_value = runtime.block_on(async {
            // 実際にはここでダウンロードの進捗を取得する予定
            manager.get_download_progress(download_id).await
        }).unwrap_or(0.5); // エラー時はデフォルト値を返す
        
        let progress = DownloadProgress {
            download_id: download_id.to_string(),
            progress: progress_value,
            downloaded_size: 1024 * 1024, // 1MBとしてダミー値を返す
            total_size: Some(1024 * 1024 * 10), // 10MBとしてダミー値を返す
            status: status_to_string(DownloadStatus::Downloading),
            error_message: None,
        };
        
        let json = serde_json::to_string(&progress)?;
        Ok(json)
    });
    
    result_to_c_string(result)
}

#[no_mangle]
pub extern "C" fn pause_download(download_id_ptr: *const c_char) -> i8 {
    let result = catch_unwind_result(|| {
        let download_id = unsafe { CStr::from_ptr(download_id_ptr).to_str()? };
        
        let manager = DOWNLOAD_MANAGER.write()
            .map_err(|e| anyhow!("ダウンロードマネージャーの取得に失敗しました: {}", e))?;
        
        let runtime = RUNTIME.lock()
            .map_err(|e| anyhow!("ランタイムの取得に失敗しました: {}", e))?;
            
        let result = runtime.block_on(async {
            // 一時停止機能が実装されていないため、キャンセル機能を使用
            manager.cancel_download(download_id).await
        });
        
        match result {
            Ok(_) => Ok(true),
            Err(e) => Err(anyhow!("ダウンロードの一時停止に失敗しました: {}", e)),
        }
    });
    
    match result {
        Ok(true) => 1,
        _ => 0,
    }
}

#[no_mangle]
pub extern "C" fn resume_download(download_id_ptr: *const c_char) -> i8 {
    let result = catch_unwind_result(|| {
        let download_id = unsafe { CStr::from_ptr(download_id_ptr).to_str()? };
        
        // 現在は再開機能が実装されていないため、常に成功を返す
        // 将来的には、以下のように実装する予定
        // let manager = DOWNLOAD_MANAGER.write()
        //     .map_err(|e| anyhow!("ダウンロードマネージャーの取得に失敗しました: {}", e))?;
        // 
        // let runtime = RUNTIME.lock().unwrap();
        // let result = runtime.block_on(async {
        //     manager.resume_download(download_id).await
        // });
        
        // スタブ実装として成功を返す
        Ok(true)
    });
    
    match result {
        Ok(true) => 1,
        _ => 0,
    }
}

#[no_mangle]
pub extern "C" fn cancel_download(download_id_ptr: *const c_char) -> i8 {
    let result = catch_unwind_result(|| {
        let download_id = unsafe { CStr::from_ptr(download_id_ptr).to_str()? };
        
        let manager = DOWNLOAD_MANAGER.write()
            .map_err(|e| anyhow!("ダウンロードマネージャーの取得に失敗しました: {}", e))?;
        
        let runtime = RUNTIME.lock()
            .map_err(|e| anyhow!("ランタイムの取得に失敗しました: {}", e))?;
            
        let result = runtime.block_on(async {
            manager.cancel_download(download_id).await
        });
        
        match result {
            Ok(_) => Ok(true),
            Err(e) => Err(anyhow!("ダウンロードのキャンセルに失敗しました: {}", e)),
        }
    });
    
    match result {
        Ok(true) => 1,
        _ => 0,
    }
}

#[no_mangle]
pub extern "C" fn get_download_list() -> *mut c_char {
    let result = catch_unwind_result(|| {
        // 現在は真のダウンロードリスト機能が実装されていないため、ダミーデータを返す
        // 将来的には、以下のように実装する予定
        // let manager = DOWNLOAD_MANAGER.read()
        //    .map_err(|e| anyhow!("ダウンロードマネージャーの取得に失敗しました: {}", e))?;
        // 
        // let downloads = manager.get_downloads();
        // let download_infos = downloads.iter().map(|download| {
        //     DownloadInfo {
        //         download_id: download.id().to_string(),
        //         url: download.url().to_string(),
        //         destination: download.destination().to_string_lossy().to_string(),
        //         status: status_to_string(download.status()),
        //         progress: download.progress(),
        //         downloaded_size: download.downloaded_size(),
        //         total_size: download.total_size(),
        //         created_at: download.created_at().to_rfc3339(),
        //         updated_at: download.updated_at().to_rfc3339(),
        //         error_message: download.error_message().map(|e| e.to_string()),
        //     }
        // }).collect::<Vec<_>>();
        
        // ダミーデータを生成
        let download_infos = vec![
            DownloadInfo {
                download_id: Uuid::new_v4().to_string(),
                url: "https://example.com/sample.mp4".to_string(),
                destination: "/Users/Downloads/sample.mp4".to_string(),
                status: status_to_string(DownloadStatus::Downloading),
                progress: 0.3,
                downloaded_size: 1024 * 1024 * 3, // 3MB
                total_size: Some(1024 * 1024 * 10), // 10MB
                created_at: Utc::now().to_rfc3339(),
                updated_at: Utc::now().to_rfc3339(),
                error_message: None,
            },
            DownloadInfo {
                download_id: Uuid::new_v4().to_string(),
                url: "https://example.com/document.pdf".to_string(),
                destination: "/Users/Downloads/document.pdf".to_string(),
                status: status_to_string(DownloadStatus::Completed),
                progress: 1.0,
                downloaded_size: 1024 * 1024 * 5, // 5MB
                total_size: Some(1024 * 1024 * 5), // 5MB
                created_at: Utc::now().to_rfc3339(),
                updated_at: Utc::now().to_rfc3339(),
                error_message: None,
            },
        ];
        
        let json = serde_json::to_string(&download_infos)?;
        Ok(json)
    });
    
    result_to_c_string(result)
}

#[no_mangle]
pub extern "C" fn detect_content_type(url_ptr: *const c_char) -> *mut c_char {
    let result = catch_unwind_result(|| {
        let url = unsafe { CStr::from_ptr(url_ptr).to_str()? };
        
        let manager = DOWNLOAD_MANAGER.read()
            .map_err(|e| anyhow!("ダウンロードマネージャーの取得に失敗しました: {}", e))?;
        
        // detect_content_typeを非同期で実行する
        let runtime = RUNTIME.lock()
            .map_err(|e| anyhow!("ランタイムの取得に失敗しました: {}", e))?;
            
        let result = runtime.block_on(async {
            manager.detect_content_type(url)
        });
        
        match result {
            Ok(content_type) => Ok(content_type_to_string(content_type)),
            Err(e) => Err(anyhow!("コンテンツタイプの検出に失敗しました: {}", e)),
        }
    });
    
    result_to_c_string(result)
}

#[no_mangle]
pub extern "C" fn get_youtube_video_info(url_ptr: *const c_char) -> *mut c_char {
    let result = catch_unwind_result(|| {
        let url = unsafe { CStr::from_ptr(url_ptr).to_str()? };
        
        // 現在はYouTube動画情報取得機能が実装されていないため、スタブ実装を提供
        // 将来的には、以下のように実装する予定
        // let manager = DOWNLOAD_MANAGER.read()
        //     .map_err(|e| anyhow!("ダウンロードマネージャーの取得に失敗しました: {}", e))?;
        // 
        // let runtime = RUNTIME.lock().unwrap();
        // let result = runtime.block_on(async {
        //     manager.get_youtube_video_info(url).await
        // });
        
        // スタブ実装としてダミーデータを返す
        let formats = vec![
            YoutubeVideoFormat {
                quality: "720p".to_string(),
                format: "mp4".to_string(),
                size: 1024 * 1024 * 10, // 10MB
            },
            YoutubeVideoFormat {
                quality: "360p".to_string(),
                format: "mp4".to_string(),
                size: 1024 * 1024 * 5, // 5MB
            },
        ];
        
        let video_info = YoutubeVideoInfo {
            title: format!("テスト動画: {}", url),
            author: "NextDownloader".to_string(),
            duration: 180, // 3分
            formats,
        };
        
        let json = serde_json::to_string(&video_info)?;
        Ok(json)
    });
    
    result_to_c_string(result)
}

#[no_mangle]
pub extern "C" fn get_settings() -> *mut c_char {
    let result = catch_unwind_result(|| {
        // 現在は設定取得機能が実装されていないため、デフォルト設定を返す
        // 将来的には、以下のように実装する予定
        // let manager = DOWNLOAD_MANAGER.read()
        //     .map_err(|e| anyhow!("ダウンロードマネージャーの取得に失敗しました: {}", e))?;
        // 
        // let settings = manager.get_settings();
        
        // デフォルト設定を返す
        let settings_wrapper = Settings {
            default_download_path: "/Users/Downloads".to_string(),
            max_concurrent_downloads: 3,
            use_system_notification: true,
            auto_extract_archives: true,
            theme: "system".to_string(),
        };
        
        let json = serde_json::to_string(&settings_wrapper)?;
        Ok(json)
    });
    
    result_to_c_string(result)
}

#[no_mangle]
pub extern "C" fn update_settings(settings_json_ptr: *const c_char) -> i8 {
    let result = catch_unwind_result(|| {
        let settings_json = unsafe { CStr::from_ptr(settings_json_ptr).to_str()? };
        
        // 設定をJSONからデシリアライズ
        let settings_wrapper: Settings = serde_json::from_str(settings_json)?;
        
        // 現在は設定更新機能が実装されていないため、成功を返すのみ
        // 将来的には、以下のように実装する予定
        // let manager = DOWNLOAD_MANAGER.write()
        //     .map_err(|e| anyhow!("ダウンロードマネージャーの取得に失敗しました: {}", e))?;
        // 
        // // 設定を更新
        // let mut settings = manager.get_settings_mut();
        // settings.default_download_path = PathBuf::from(&settings_wrapper.default_download_path);
        // settings.max_concurrent_downloads = settings_wrapper.max_concurrent_downloads as usize;
        // settings.use_system_notification = settings_wrapper.use_system_notification;
        // settings.auto_extract_archives = settings_wrapper.auto_extract_archives;
        // settings.theme = settings_wrapper.theme.clone();
        // 
        // // 設定を保存
        // manager.save_settings()?;
        
        // ログに設定を出力するのみ
        info!("設定が更新されました: {:?}", settings_wrapper);
        
        Ok(true)
    });
    
    match result {
        Ok(true) => 1,
        _ => 0,
    }
}

// ユーティリティ関数

// パニックをキャッチして結果を返す
fn catch_unwind_result<F, T>(f: F) -> Result<T>
where
    F: FnOnce() -> Result<T> + std::panic::UnwindSafe,
{
    match std::panic::catch_unwind(f) {
        Ok(result) => result,
        Err(e) => {
            let error_message = if let Some(s) = e.downcast_ref::<&str>() {
                format!("パニックが発生しました: {}", s)
            } else if let Some(s) = e.downcast_ref::<String>() {
                format!("パニックが発生しました: {}", s)
            } else {
                "不明なパニックが発生しました".to_string()
            };
            Err(anyhow!(error_message))
        }
    }
}

// 結果をC文字列に変換
fn result_to_c_string<T: serde::Serialize>(result: Result<T>) -> *mut c_char {
    match result {
        Ok(value) => {
            match serde_json::to_string(&value) {
                Ok(json) => {
                    match CString::new(json) {
                        Ok(c_str) => c_str.into_raw(),
                        Err(_) => CString::new(r#"{"error":"CStringへの変換に失敗しました"}"#).unwrap().into_raw(),
                    }
                },
                Err(e) => {
                    let error_json = format!(r#"{{"error":"JSONへのシリアライズに失敗しました: {}"}}"#, e);
                    CString::new(error_json).unwrap().into_raw()
                }
            }
        },
        Err(e) => {
            let error_json = format!(r#"{{"error":"{}"}}"#, e);
            CString::new(error_json).unwrap().into_raw()
        }
    }
}

// ロガーの初期化
fn init_logger() -> Result<()> {
    // ここにロガーの初期化コードを実装
    // 例: env_logger::init();
    Ok(())
}

// ダウンロードステータスを文字列に変換
fn status_to_string(status: DownloadStatus) -> String {
    match status {
        DownloadStatus::Initializing => "initializing".to_string(),
        DownloadStatus::Downloading => "downloading".to_string(),
        DownloadStatus::Paused => "paused".to_string(),
        DownloadStatus::Completed => "completed".to_string(),
        DownloadStatus::Error => "error".to_string(),
        DownloadStatus::Cancelled => "cancelled".to_string(),
    }
}

// コンテンツタイプを文字列に変換
fn content_type_to_string(content_type: ContentType) -> String {
    match content_type {
        ContentType::MP4 => "video/mp4".to_string(),
        ContentType::HLS => "application/x-mpegURL".to_string(),
        ContentType::DASH => "application/dash+xml".to_string(),
        ContentType::YouTube => "youtube".to_string(),
        ContentType::Unknown => "application/octet-stream".to_string(),
    }
}