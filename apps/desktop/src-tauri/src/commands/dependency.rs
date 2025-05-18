//! # 依存関係チェックコマンド
//! 
//! 依存関係のチェックを行うTauriコマンドを提供します。

use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

/// 依存関係のステータス
#[derive(Debug, Serialize)]
pub struct DependencyStatus {
    /// yt-dlpがインストールされているか
    pub ytdlp: bool,
    /// aria2cがインストールされているか
    pub aria2c: bool,
    /// ffmpegがインストールされているか
    pub ffmpeg: bool,
}

/// 依存関係をチェックします
#[tauri::command]
pub async fn check_dependencies(
    app_state: State<'_, AppState>,
) -> Result<DependencyStatus, String> {
    info!("依存関係をチェックしています...");
    
    let download_manager = app_state.download_manager.lock().unwrap();
    
    // 依存関係をチェック
    match download_manager.check_dependencies().await {
        Ok(status) => {
            Ok(DependencyStatus {
                ytdlp: status.ytdlp,
                aria2c: status.aria2c,
                ffmpeg: status.ffmpeg,
            })
        },
        Err(e) => Err(format!("依存関係のチェックに失敗しました: {}", e)),
    }
}