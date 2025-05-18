//! # 設定コマンド
//! 
//! アプリケーション設定を管理するTauriコマンドを提供します。

use std::path::PathBuf;

use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use tauri::State;

use nextdownloader_core::AppConfig;

use crate::state::AppState;

/// 設定取得レスポンス
#[derive(Debug, Serialize)]
pub struct GetConfigResponse {
    /// 設定内容
    pub config: AppConfig,
    /// 設定ファイルのパス
    pub config_path: String,
}

/// 設定を取得します
#[tauri::command]
pub async fn get_config(
    app_state: State<'_, AppState>,
) -> Result<GetConfigResponse, String> {
    info!("アプリケーション設定を取得しています...");
    
    let config = app_state.config.read().await.clone();
    let config_path = AppConfig::default_config_path();
    
    Ok(GetConfigResponse {
        config,
        config_path: config_path.to_string_lossy().to_string(),
    })
}

/// 設定保存リクエスト
#[derive(Debug, Deserialize)]
pub struct SaveConfigRequest {
    /// 設定内容
    pub config: AppConfig,
}

/// 設定を保存します
#[tauri::command]
pub async fn save_config(
    app_state: State<'_, AppState>,
    request: SaveConfigRequest,
) -> Result<(), String> {
    info!("アプリケーション設定を保存しています...");
    
    // 設定を更新
    {
        let mut config = app_state.config.write().await;
        *config = request.config.clone();
    }
    
    // 設定ファイルのパスを取得
    let config_path = AppConfig::default_config_path();
    
    // 設定を保存
    match request.config.save(&config_path) {
        Ok(_) => {
            info!("設定を保存しました: {:?}", config_path);
            Ok(())
        },
        Err(e) => {
            error!("設定の保存に失敗しました: {}", e);
            Err(format!("設定の保存に失敗しました: {}", e))
        },
    }
}