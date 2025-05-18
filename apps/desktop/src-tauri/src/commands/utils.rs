//! # ユーティリティコマンド
//! 
//! ユーティリティ機能を提供するTauriコマンドです。

use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use tauri::{Manager, State};
use tauri_plugin_dialog::DialogExt;

use crate::state::AppState;

/// ディレクトリ選択リクエスト
#[derive(Debug, Deserialize)]
pub struct SelectDirectoryRequest {
    /// ダイアログのタイトル
    pub title: Option<String>,
    /// 初期ディレクトリ
    pub default_path: Option<String>,
}

/// ディレクトリ選択レスポンス
#[derive(Debug, Serialize)]
pub struct SelectDirectoryResponse {
    /// 選択されたディレクトリのパス
    pub path: Option<String>,
}

/// ディレクトリを選択します
#[tauri::command]
pub async fn select_directory(
    app: tauri::AppHandle,
    request: SelectDirectoryRequest,
) -> Result<SelectDirectoryResponse, String> {
    info!("ディレクトリ選択ダイアログを表示します");
    
    let mut dialog = app.dialog();
    
    // タイトルを設定
    if let Some(title) = request.title {
        dialog = dialog.title(&title);
    }
    
    // 初期ディレクトリを設定
    if let Some(default_path) = request.default_path {
        dialog = dialog.default_path(default_path);
    }
    
    // ディレクトリ選択ダイアログを表示
    match dialog.select_directory() {
        Ok(path) => {
            Ok(SelectDirectoryResponse {
                path: path.map(|p| p.to_string_lossy().to_string()),
            })
        },
        Err(e) => {
            error!("ディレクトリ選択ダイアログの表示に失敗しました: {}", e);
            Err(format!("ディレクトリ選択ダイアログの表示に失敗しました: {}", e))
        },
    }
}

/// バージョン情報を取得します
#[tauri::command]
pub fn get_version() -> String {
    nextdownloader_core::version()
}