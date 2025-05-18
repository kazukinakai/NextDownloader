//! # ダウンロードコマンド
//! 
//! ダウンロード関連のTauriコマンドを提供します。

use std::path::PathBuf;

use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use tauri::State;

use nextdownloader_core::{ContentType, DownloadOptions};

use crate::state::{AppState, DownloadInfo};

/// ダウンロード開始リクエスト
#[derive(Debug, Deserialize)]
pub struct StartDownloadRequest {
    /// ダウンロードURL
    pub url: String,
    /// 保存先ディレクトリ
    pub destination_dir: String,
    /// ファイル名（省略可）
    pub filename: Option<String>,
    /// フォーマット（省略可）
    pub format: Option<String>,
}

/// ダウンロード開始レスポンス
#[derive(Debug, Serialize)]
pub struct StartDownloadResponse {
    /// ダウンロードID
    pub id: String,
}

/// ダウンロードを開始します
#[tauri::command]
pub async fn start_download(
    app_state: State<'_, AppState>,
    request: StartDownloadRequest,
) -> Result<StartDownloadResponse, String> {
    info!("ダウンロードを開始します: {}", request.url);
    
    let download_manager = app_state.download_manager.lock().unwrap();
    
    // 保存先パスを構築
    let mut destination_path = PathBuf::from(&request.destination_dir);
    
    // ファイル名が指定されている場合は追加
    if let Some(filename) = &request.filename {
        destination_path = destination_path.join(filename);
    }
    
    // コンテンツタイプを検出
    let content_type = match download_manager.detect_content_type(&request.url) {
        Ok(content_type) => content_type,
        Err(e) => {
            error!("コンテンツタイプの検出に失敗しました: {}", e);
            ContentType::Unknown
        }
    };
    
    // ダウンロードを開始
    let download_id = match download_manager.start_download(
        &request.url,
        &destination_path,
        request.format.as_deref(),
    ).await {
        Ok(id) => id,
        Err(e) => return Err(format!("ダウンロードの開始に失敗しました: {}", e)),
    };
    
    // ダウンロード情報を追加
    app_state.add_download(
        download_id.clone(),
        request.url,
        destination_path.to_string_lossy().to_string(),
        format!("{:?}", content_type),
    ).await;
    
    Ok(StartDownloadResponse {
        id: download_id,
    })
}

/// ダウンロード進捗リクエスト
#[derive(Debug, Deserialize)]
pub struct GetDownloadProgressRequest {
    /// ダウンロードID
    pub id: String,
}

/// ダウンロード進捗を取得します
#[tauri::command]
pub async fn get_download_progress(
    app_state: State<'_, AppState>,
    request: GetDownloadProgressRequest,
) -> Result<DownloadInfo, String> {
    let download_manager = app_state.download_manager.lock().unwrap();
    
    // ダウンロードの進捗を取得
    match download_manager.get_download_progress(&request.id).await {
        Ok(progress) => {
            // ダウンロード情報を更新
            app_state.update_download(&request.id, progress).await;
            
            // 最新のダウンロード情報を取得
            let downloads = app_state.downloads.read().await;
            if let Some(download) = downloads.get(&request.id) {
                Ok(download.clone())
            } else {
                Err(format!("ダウンロードIDが見つかりません: {}", request.id))
            }
        },
        Err(e) => Err(format!("ダウンロード進捗の取得に失敗しました: {}", e)),
    }
}

/// ダウンロードキャンセルリクエスト
#[derive(Debug, Deserialize)]
pub struct CancelDownloadRequest {
    /// ダウンロードID
    pub id: String,
}

/// ダウンロードをキャンセルします
#[tauri::command]
pub async fn cancel_download(
    app_state: State<'_, AppState>,
    request: CancelDownloadRequest,
) -> Result<(), String> {
    info!("ダウンロードをキャンセルします: {}", request.id);
    
    let download_manager = app_state.download_manager.lock().unwrap();
    
    // ダウンロードをキャンセル
    match download_manager.cancel_download(&request.id).await {
        Ok(_) => {
            // ダウンロード情報を削除
            app_state.remove_download(&request.id).await;
            Ok(())
        },
        Err(e) => Err(format!("ダウンロードのキャンセルに失敗しました: {}", e)),
    }
}

/// ダウンロード一覧を取得します
#[tauri::command]
pub async fn get_downloads(
    app_state: State<'_, AppState>,
) -> Result<Vec<DownloadInfo>, String> {
    // 全てのダウンロード情報を取得
    let downloads = app_state.get_downloads().await;
    Ok(downloads)
}